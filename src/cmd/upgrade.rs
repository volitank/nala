use std::collections::VecDeque;
use std::env;
use std::ffi::CString;
use std::io::{BufWriter, Write};
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
use std::path::Path;
use std::process::Command;

use anyhow::{bail, Result};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{close, dup2, execv, fork, pipe, ForkResult};
use rust_apt::cache::Upgrade;
use rust_apt::raw::quote_string;
use rust_apt::{Marked, new_cache, Package, PkgCurrentState, Version};

use crate::config::Paths;
use crate::util::{get_pkg_name, sudo_check};
use crate::{debug, Config};

/// The subset of APT pre-install hook actions that Nala currently models.
enum HookActionKind {
	/// A package will be unpacked from the given `.deb` path.
	Install { filename: String },
	/// A package is only being configured, not unpacked or removed.
	Configure,
	/// A package will be removed or purged.
	Remove,
}

/// A single hook-facing package action derived from the current depcache
/// marks.
struct HookAction<'a> {
	package: Package<'a>,
	kind: HookActionKind,
}

/// Executes an upgrade transaction after applying the selected APT upgrade
/// mode to a fresh cache.
pub async fn upgrade(config: &Config, upgrade_type: Upgrade) -> Result<()> {
	sudo_check(config)?;
	let cache = new_cache!()?;

	debug!("Running Upgrade: {upgrade_type:?}");
	cache.upgrade(upgrade_type)?;

	crate::summary::commit(cache, config).await
}

/// Runs each configured shell hook under `key` and aborts on the first
/// non-zero exit status.
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

/// Returns the APT hook comparison marker between the currently installed
/// version and the target install version.
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

/// Formats the architecture and Multi-Arch fields used by protocol version 3
/// hook payloads.
fn set_multi_arch(version: &Version, hook_ver: i32) -> String {
	if hook_ver < 3 {
		return String::new();
	}

	format!("{} {} ", version.arch(), version.multi_arch_type())
}

/// Looks up the special `now` version used by APT when a package is being
/// removed without an installed candidate entry.
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

/// Resolves the filename that should be reported to pre-install hooks for an
/// install-like action.
fn install_filename(pkg: &Package, archive: &Path) -> Option<String> {
	let version = pkg.candidate().or_else(|| pkg.installed()).or_else(|| get_now_version(pkg))?;
	let filename_record = version.get_record(rust_apt::records::RecordField::Filename)?;

	if filename_record.starts_with('/') {
		return Some(filename_record);
	}

	Some(archive.join(get_pkg_name(&version)).display().to_string())
}

/// Converts the current depcache mark for `pkg` into a hook action when that
/// package should be visible to `DPkg::Pre-Install-Pkgs`.
fn hook_action<'a>(pkg: &Package<'a>, archive: &Path) -> Option<HookAction<'a>> {
	match pkg.marked() {
		Marked::NewInstall
		| Marked::Install
		| Marked::Upgrade
		| Marked::Downgrade
		| Marked::ReInstall => Some(HookAction {
			package: pkg.clone(),
			kind: HookActionKind::Install {
				filename: install_filename(pkg, archive)?,
			},
		}),
		Marked::Remove | Marked::Purge => Some(HookAction {
			package: pkg.clone(),
			kind: HookActionKind::Remove,
		}),
		Marked::Keep if pkg.current_state() == PkgCurrentState::ConfigFiles => Some(HookAction {
			package: pkg.clone(),
			kind: HookActionKind::Configure,
		}),
		_ => None,
	}
}

/// Serializes a single hook action into the versioned `Pre-Install-Pkgs`
/// protocol line consumed by tools like `apt-listchanges`.
fn pkg_info(action: &HookAction<'_>, hook_ver: i32) -> String {
	let mut string = String::new();
	let pkg = &action.package;

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

	match &action.kind {
		HookActionKind::Install { filename } => {
			string += filename;
			string.push('\n');
		},
		HookActionKind::Configure => string += "**CONFIGURE**\n",
		HookActionKind::Remove => string += "**REMOVE**\n",
	}

	string
}

/// Writes the protocol header and escaped APT configuration tree for versioned
/// pre-install hooks.
fn write_config_info<W: Write>(w: &mut W, config: &Config, hook_ver: i32) -> Result<()> {
	let Some(tree) = config.apt.root_tree() else {
		bail!("No config tree!");
	};

	if hook_ver <= 3 {
		writeln!(w, "VERSION {hook_ver}")?;
	} else {
		writeln!(w, "VERSION 3")?;
	}

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
			}
		}
	}
	writeln!(w)?;
	Ok(())
}

/// Runs `DPkg::Pre-Install-Pkgs`-style hooks and feeds them package action
/// metadata using the APT hook protocol version they requested.
pub fn apt_hook_with_pkgs(config: &Config, pkgs: &Vec<Package>, key: &str) -> Result<()> {
	let archive = config.get_path(&Paths::Archive);
	let actions: Vec<_> = pkgs
		.iter()
		.filter_map(|pkg| hook_action(pkg, &archive))
		.collect();

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

		debug!("Forking Child for '{hook}'");
		let (statusfd, writefd) = pipe()?;

		match unsafe { fork()? } {
			ForkResult::Child => {
				close(writefd.as_raw_fd())?;
				dup2(statusfd.as_raw_fd(), info_fd)?;

				debug!("From Child");
				env::set_var("APT_HOOK_INFO_FD", info_fd.to_string());
				if key == "DPkg::Pre-Install-Pkgs" {
					env::set_var("DPKG_FRONTEND_LOCKED", "true");
				}

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
				let file = unsafe { std::fs::File::from_raw_fd(writefd.into_raw_fd()) };
				let mut w = BufWriter::new(file);

				if hook_ver >= 2 {
					write_config_info(&mut w, config, hook_ver)?;
					debug!("Writing action data into child");
					for action in &actions {
						let line = pkg_info(action, hook_ver);
						debug!("{line}");
						write!(w, "{line}")?;
					}
				} else {
					debug!("Writing install filenames into child");
					for action in &actions {
						let HookActionKind::Install { filename } = &action.kind else {
							continue;
						};
						debug!("{filename}");
						writeln!(w, "{filename}")?;
					}
				}
				w.flush()?;
				// Must drop the pipe or the child may hang
				drop(w);
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
	if resp.trim().is_empty() || resp.starts_with('y') {
		return Ok(());
	}

	if resp.starts_with('n') {
		bail!("User refused confirmation")
	}

	bail!("'{}' is not a valid response", response.trim())
}
