use std::collections::VecDeque;
use std::env;
use std::ffi::CString;
use std::io::Write;
use std::os::fd::{AsRawFd, FromRawFd};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup2, execv, fork, pipe, ForkResult};
use rust_apt::cache::Upgrade;
use rust_apt::raw::quote_string;
use rust_apt::{new_cache, Package, PkgCurrentState, Version};

use crate::config::Paths;
use crate::util::{get_pkg_name, sudo_check};
use crate::{debug, Config};

pub async fn upgrade(config: &Config, upgrade_type: Upgrade) -> Result<()> {
	sudo_check(config)?;
	let cache = new_cache!()?;

	debug!("Running Upgrade: {upgrade_type:?}");
	cache.upgrade(upgrade_type)?;

	crate::summary::commit(cache, config).await
}

pub fn run_scripts(config: &Config, key: &str) -> Result<()> {
	for hook in config.apt.find_vector(key) {
		debug!("Running {hook}");
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

	format!("{} {} ", version.arch(), version.multi_arch_type())
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

		if let (Some(tag), Some(value)) = (node.full_tag(), node.value()) {
			if !value.is_empty() {
				let tag_value = format!(
					"{}={}",
					quote_string(&tag, "=\"\n".to_string()),
					quote_string(&value, "\n".to_string())
				);
				debug!("{tag_value}");
				writeln!(w, "{tag_value}",)?;
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

		debug!("{prog} is version {hook_ver} on fd {info_fd}");

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

			let filename = archive.join(get_pkg_name(&cand));
			if !filename.exists() {
				continue;
			}
			hook_strings.push(format!("{}\n", filename.display()))
		}

		debug!("Forking Child for '{hook}'");
		let (statusfd, writefd) = pipe()?;

		match unsafe { fork()? } {
			ForkResult::Child => {
				close(writefd.as_raw_fd())?;
				dup2(statusfd.as_raw_fd(), info_fd)?;

				debug!("From Child");
				env::set_var("APT_HOOK_INFO_FD", info_fd.to_string());

				let mut args_cstr: Vec<CString> = vec![];
				for arg in ["/bin/sh", "-c", &hook] {
					args_cstr.push(CString::new(arg)?)
				}
				debug!("Exec {args_cstr:?}");
				execv(&args_cstr[0], &args_cstr)?;

				// Ensure exit after execv if it fails
				std::process::exit(1);
			},
			ForkResult::Parent { child } => {
				let mut w = unsafe { std::fs::File::from_raw_fd(writefd.as_raw_fd()) };

				if hook_ver >= 2 {
					write_config_info(&mut w, config, hook_ver)?;
				}

				debug!("Writing data into child");
				for pkg in hook_strings {
					debug!("{pkg}");
					write!(w, "{pkg}")?;
					w.flush()?;
				}
				// Must drop the pipe or the child may hang
				drop(w);
				// Forget the file descriptor as we just closed it with drop
				std::mem::forget(writefd);
				debug!("Waiting for Child");

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
