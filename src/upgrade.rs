use std::collections::{HashMap, VecDeque};
use std::env;
use std::ffi::CString;
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};
use chrono::Utc;
use nix::fcntl::{fcntl, FcntlArg, FdFlag, OFlag};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup2, execv, fork, pipe, ForkResult};
use rust_apt::cache::Upgrade;
use rust_apt::raw::quote_string;
use rust_apt::{new_cache, Cache, Marked, Package, PkgCurrentState, Version};

use crate::config::Paths;
use crate::downloader::Downloader;
use crate::history::{self, HistoryEntry, HistoryPackage, Operation};
use crate::util::{get_pkg_name, sudo_check};
use crate::{dpkg, dprint, table, tui, Config};

pub fn auto_remover(cache: &Cache) -> Vec<Version> {
	let mut marked_remove = vec![];
	for package in cache.iter() {
		if !package.is_auto_removable() {
			continue;
		}

		if package.current_state() != PkgCurrentState::ConfigFiles {
			package.mark_delete(false);
			if let Some(inst) = package.installed() {
				marked_remove.push(inst);
			}
		} else {
			package.mark_keep();
		}
	}
	// There is more code in private-install.cc DoAutomaticremove
	// If there are auto_remove bugs consider implementing that.
	marked_remove
}

#[tokio::main]
pub async fn upgrade(config: &Config) -> Result<()> {
	sudo_check(config)?;
	let cache = new_cache!()?;

	// TODO: This whole section is slow. It iters the entire cache several times.
	// Need to optimize it by doing one iteration, and this will include
	// NOT using the cache.get_changes method.

	// SafeUpgrade takes precedence.
	let upgrade_type = if config.get_bool("safe", false) {
		Upgrade::SafeUpgrade
	} else if config.full_upgrade() {
		Upgrade::FullUpgrade
	} else {
		Upgrade::Upgrade
	};

	dprint!(config, "Running Upgrade: {upgrade_type:?}");
	cache.upgrade(upgrade_type)?;

	dprint!(config, "Running auto_remover");
	let auto_remove = auto_remover(&cache);
	let mut pkg_set: HashMap<Operation, Vec<HistoryPackage>> = HashMap::new();

	dprint!(config, "Calculating changes");
	let changed = cache.get_changes(true).collect::<Vec<_>>();

	for pkg in &changed {
		let (op, ver) = match pkg.marked() {
			mark @ (Marked::NewInstall
			| Marked::Install
			| Marked::ReInstall
			| Marked::Downgrade) => {
				let Some(cand) = pkg.install_version() else {
					continue;
				};
				let op = match mark {
					Marked::ReInstall => Operation::Reinstall,
					Marked::Downgrade => Operation::Downgrade,
					_ => Operation::Install,
				};
				(op, cand)
			},
			Marked::Remove | Marked::Purge => {
				let Some(inst) = pkg.installed() else {
					continue;
				};

				if auto_remove.contains(&inst) {
					continue;
				}

				let op = if pkg.marked_purge() { Operation::Purge } else { Operation::Remove };
				(op, inst)
			},
			Marked::Upgrade => {
				if let (Some(inst), Some(cand)) = (pkg.installed(), pkg.candidate()) {
					pkg_set.entry(Operation::Upgrade).or_default().push(
						HistoryPackage::from_version(Operation::Upgrade, &cand, &Some(inst)),
					);
				}
				continue;
			},
			// TODO: See if pkg is held for phasing and show percent
			// pkgDepCache::PhasingApplied
			// VerIterator::PhasedUpdatePercentage
			Marked::Held => {
				let Some(cand) = pkg.candidate() else {
					continue;
				};
				(Operation::Held, cand)
			},
			Marked::Keep => continue,
			Marked::None => bail!("{pkg} not marked, this should be impossible"),
		};

		pkg_set
			.entry(op)
			.or_default()
			.push(HistoryPackage::from_version(op, &ver, &None));
	}

	if !auto_remove.is_empty() {
		pkg_set.insert(
			Operation::AutoRemove,
			auto_remove
				.into_iter()
				.map(|v| HistoryPackage::from_version(Operation::AutoRemove, &v, &None))
				.collect(),
		);
	}

	let versions = changed
		.iter()
		.filter_map(|pkg| pkg.install_version())
		.collect::<Vec<_>>();

	let mut downloader = Downloader::new(config)?;
	for ver in &versions {
		downloader.add_version(ver, config)?;
	}

	if config.get_bool("print_uris", false) {
		for uri in downloader.uris() {
			println!("{}", serde_json::to_string_pretty(uri)?);
		}
		// Print uris does not go past here
		return Ok(());
	};

	// TODO: Implement a simple summary that is very short for serial/console users

	if config.show_tui() {
		// App returns true if we should continue.
		if !tui::summary::SummaryTab::new(&cache, config, &pkg_set)
			.run()
			.await?
		{
			return Ok(());
		}
	} else {
		let mut tables = vec![];

		for (op, pkgs) in &pkg_set {
			let mut table = table::get_table(
				config,
				if pkgs[0].items(config).len() > 3 {
					&["Package:", "Old Version:", "New Version:", "Size:"]
				} else {
					&["Package:", "Version:", "Size:"]
				},
			);

			table.add_rows(pkgs.iter().map(|p| p.items(config)));
			tables.push((op, table));
		}

		for (op, pkgs) in tables {
			let width = rust_apt::util::terminal_width();
			let sep = "=".repeat(width);
			println!("{sep}");
			println!(" {}", config.highlight(op.as_str()));
			println!("{sep}");

			println!("{pkgs}");
		}

		// Returns an error if yes is no selected
		ask("Do you want to continue?")?;
	}

	let history_entry = HistoryEntry::new(
		history::get_history(config)?
			.iter()
			.map(|entry| entry.id)
			.max()
			.unwrap_or_default()
			+ 1,
		Utc::now().to_rfc3339(),
		pkg_set.into_values().flatten().collect(),
	);

	history_entry.write_to_file(config)?;

	bincode::serialize(&history_entry)?;

	// TODO: download only mode?
	// TODO: Need to handle Cntrl C here.
	// This will start run_scripts below sometimes if you hit it.
	let _finished = downloader.run(config, false).await?;

	run_scripts(config, "DPkg::Pre-Invoke")?;
	apt_hook_with_pkgs(config, &changed, "DPkg::Pre-Install-Pkgs")?;

	config.apt.set("Dpkg::Use-Pty", "0");

	dpkg::run_install(cache, config)?;

	run_scripts(config, "DPkg::Post-Invoke")?;
	Ok(())
}

pub fn run_scripts(config: &Config, key: &str) -> Result<()> {
	for hook in config.apt.find_vector(key) {
		dprint!(config, "Running {hook}");
		let mut child = Command::new("sh").arg("-c").arg(hook).spawn()?;

		let exit = child.wait()?;
		if !exit.success() {
			// TODO: Figure out how to return the ExitStatus from main.
			std::process::exit(exit.code().unwrap());
		}
	}
	config.apt.clear(key);
	Ok(())
}

/// Set the compare string.
fn set_comp<'a>(current: &Option<Version<'a>>, cand: &Version<'a>) -> &'static str {
	let Some(current) = current else {
		return "<";
	};

	match current.cmp(cand) {
		std::cmp::Ordering::Less => "<",
		std::cmp::Ordering::Equal => "=",
		std::cmp::Ordering::Greater => ">",
	}
}

/// Set multi archi if hook version is 3.
fn set_multi_arch(version: &Version, hook_ver: i32) -> String {
	if hook_ver < 3 {
		return String::new();
	}

	format!("{} {} ", version.arch(), version.multi_arch())
}

fn get_now_version<'a>(pkg: &Package<'a>) -> Option<Version<'a>> {
	for ver in pkg.versions() {
		for pkg_file in ver.package_files() {
			if let Some(archive) = pkg_file.archive() {
				if archive == "now" {
					return Some(ver);
				}
			}
		}
	}
	None
}

pub fn pkg_info(pkg: &Package, hook_ver: i32, archive: &Path) -> String {
	let mut string = String::new();

	let current_version = pkg.installed().or_else(|| get_now_version(pkg));

	string.push_str(pkg.name());
	string.push(' ');

	if let Some(ver) = current_version.as_ref() {
		string += &format!("{} {}", ver.version(), set_multi_arch(ver, hook_ver));
	} else {
		string += if hook_ver < 3 { "- " } else { "- - none " }
	}

	if let Some(cand) = pkg.candidate() {
		string += &format!(
			"{} {} {}",
			set_comp(&current_version, &cand),
			cand.version(),
			set_multi_arch(&cand, hook_ver),
		);
	} else {
		string += if hook_ver < 3 { "> - " } else { "> - - none " }
	}

	if pkg.marked_install() || pkg.marked_upgrade() {
		string += &pkg
			.candidate()
			.as_ref()
			.or(current_version.as_ref())
			.map(get_pkg_name)
			.map_or("**ERROR**\n".to_string(), |filename| {
				format!("{}\n", archive.join(filename).display())
			});
	} else if pkg.marked_delete() {
		string += "**REMOVE**\n";
	} else if pkg.current_state() == PkgCurrentState::ConfigFiles {
		string += "**CONFIGURE**\n";
	} else {
		string += &format!("{}\n", pkg.marked_upgrade());
	}
	string
}

fn write_config_info<W: Write>(w: &mut W, config: &Config, hook_ver: i32) -> Result<()> {
	let Some(tree) = config.apt.root_tree() else {
		bail!("No config tree!");
	};

	if hook_ver <= 3 {
		writeln!(w, "VERSION {hook_ver}")?;
	} else {
		writeln!(w, "VERSION 3")?;
	}
	w.flush()?;

	let mut stack = VecDeque::new();
	stack.push_back(tree);

	while let Some(node) = stack.pop_back() {
		if let Some(item) = node.sibling() {
			stack.push_back(item);
		}

		if let Some(item) = node.child() {
			stack.push_back(item);
		}

		if let (Some(tag), Some(value)) = (node.tag(), node.value()) {
			if !value.is_empty() {
				writeln!(
					w,
					"{}={}",
					quote_string(&tag, "=\"\n".to_string()),
					quote_string(&value, "\n".to_string()),
				)?;
				w.flush()?;
			}
		}
	}
	writeln!(w)?;
	w.flush()?;
	Ok(())
}

pub fn apt_hook_with_pkgs(config: &Config, pkgs: &Vec<Package>, key: &str) -> Result<()> {
	let archive = config.get_path(&Paths::Archive);
	for hook in config.apt.find_vector(key) {
		let Some(prog) = hook.split_whitespace().next() else {
			continue;
		};

		let hook_ver = config
			.apt
			.int(&format!("DPkg::Tools::Options::{prog}::VERSION"), 1);

		let info_fd = config
			.apt
			.int(&format!("DPkg::Tools::Options::{prog}::InfoFD"), 0);

		dprint!(config, "{prog} is version {hook_ver} on fd {info_fd}");

		let mut hook_strings: Vec<String> = vec![];

		for pkg in pkgs {
			if hook_ver > 1 {
				hook_strings.push(pkg_info(pkg, hook_ver, &archive));
				continue;
			}

			if !pkg.marked_install() || !pkg.marked_upgrade() {
				continue;
			}

			let Some(cand) = pkg.candidate() else {
				continue;
			};

			let mut filename = archive.to_owned();
			filename.push(get_pkg_name(&cand));

			if !filename.exists() {
				continue;
			}
			hook_strings.push(format!("{}\n", filename.display()))
		}

		let (statusfd, writefd) = pipe()?;

		match unsafe { fork()? } {
			ForkResult::Child => {
				set_inheritable(statusfd.as_raw_fd())?;
				dup2(statusfd.as_raw_fd(), info_fd)?;

				fcntl(statusfd.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?;
				env::set_var("APT_HOOK_INFO_FD", info_fd.to_string());

				let mut args_cstr: Vec<CString> = vec![];
				for arg in ["/bin/sh", "-c", &hook] {
					args_cstr.push(CString::new(arg)?)
				}
				execv(&args_cstr[0], &args_cstr)?;
				// Ensure exit after execv if it fails
				std::process::exit(1);
			},
			ForkResult::Parent { child } => {
				let mut w = unsafe { std::fs::File::from_raw_fd(writefd.as_raw_fd()) };

				if hook_ver >= 2 {
					write_config_info(&mut w, config, hook_ver)?;
				}

				for pkg in hook_strings {
					write!(w, "{pkg}")?;
					w.flush()?;
				}

				// Close the write end of the pipe
				close(writefd.as_raw_fd())?;

				// Wait for the child process to finish and get its exit code
				let wait_status = waitpid(child, None)?;
				if let WaitStatus::Exited(_, exit_code) = wait_status {
					if exit_code != 0 {
						std::process::exit(exit_code);
					}
				}
			},
		}
	}

	config.apt.clear(key);
	Ok(())
}

/// Ask the user a question and let them decide Y or N
pub fn ask(msg: &str) -> Result<()> {
	print!("{msg} [Y/n] ");
	std::io::stdout().flush()?;

	let mut response = String::new();
	std::io::stdin().read_line(&mut response)?;

	let resp = response.to_lowercase();
	if resp.starts_with('y') {
		return Ok(());
	}

	if resp.starts_with('n') {
		bail!("User refused confirmation")
	}

	bail!("'{}' is not a valid response", response.trim())
}

pub fn set_inheritable(fd: RawFd) -> Result<()> {
	let flags = FdFlag::from_bits_truncate(fcntl(fd, FcntlArg::F_GETFD)?);
	fcntl(fd, FcntlArg::F_SETFD(flags & !FdFlag::FD_CLOEXEC))?;
	Ok(())
}
