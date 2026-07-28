use std::fmt;
use std::fs::File;
use std::io::{ErrorKind, Read, Write, stdout};
use std::os::fd::{AsRawFd, IntoRawFd, OwnedFd, RawFd};

use anyhow::{Context, Result, anyhow, bail};
use mio::event::Iter;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use nix::fcntl::{FcntlArg, OFlag, fcntl};
use nix::libc::{TIOCGWINSZ, TIOCSWINSZ, winsize};
use nix::pty::forkpty;
use nix::sys::signal::{self, SigHandler};
use nix::sys::wait::{WaitStatus, waitpid};
use nix::unistd::{close, dup, pipe};
use nix::{ioctl_read_bad, ioctl_write_ptr_bad};
use regex::RegexBuilder;
use rust_apt::Cache;
use rust_apt::progress::{AcquireProgress, InstallProgress};

use crate::config::{Config, Theme, color};
use crate::progress::Progress;
use crate::{debug, dprog, t};

// const CURSER_UP: &'static str = "\x1b[1A";
// const CURSER_DOWN: &'static str = "\x1b[1B";
// const CURSER_FORWARD: &'static str = "\x1b[1C";
// const CURSER_BACK: &'static str = "\x1b[1D";
// const CLEAR_LINE: &'static str = "\x1b[2k";
// const CLEAR: &'static str = "\x1b[2J";
// const CLEAR_FROM_CURRENT_TO_END: &'static str = "\x1b[K";
// const BACKSPACE: &'static str = "\x08";
// const HOME: &'static str = "\x1b[H";
const ENABLE_BRACKETED_PASTE: &str = "\x1b[?2004h";
// const DISABLE_BRACKETED_PASTE: &'static str = "\x1b[?2004l";
const ENABLE_ALT_SCREEN: &str = "\x1b[?1049h";
const DISABLE_ALT_SCREEN: &str = "\x1b[?1049l";
// const SHOW_CURSOR: &'static str = "\x1b[?25h";
// const HIDE_CURSOR: &'static str = "\x1b[?25l";
// const SET_CURSER: &'static str = "\x1b[?1l";
const SAVE_TERM: &str = "\x1b[22;0;0t";
const RESTORE_TERM: &str = "\x1b[23;0;0t";
// const APPLICATION_KEYPAD: &'static str = "\x1b=";
// const NORMAL_KEYPAD: &'static str = "\x1b>";
// const CR: &'static str = "\r";
// const LF: &'static str = "\n";
// const CRLF: &'static str = "\r\n";

static mut CHILD_FD: i32 = 0;
const STDIN_FD: i32 = 0;

// Define the ioctl read call for TIOCGWINSZ
ioctl_read_bad!(tiocgwinsz, TIOCGWINSZ, winsize);
// Define the ioctl write call for TIOCSWINSZ
ioctl_write_ptr_bad!(tiocswinsz, TIOCSWINSZ, winsize);

/// Get Terminal Size from stdin
unsafe fn get_winsize() -> winsize {
	let mut ws = winsize {
		ws_row: 24,
		ws_col: 80,
		ws_xpixel: 0,
		ws_ypixel: 0,
	};
	let _ = unsafe { tiocgwinsz(STDIN_FD, &mut ws) };
	ws
}

extern "C" fn sigwinch_passthrough(_: i32) {
	unsafe {
		// Get Terminal Size from stdin.
		let ws = get_winsize();
		// Set Terminal Size for pty.
		let _ = tiocswinsz(CHILD_FD, &ws);
	}
}

struct SigwinchGuard(SigHandler);

impl SigwinchGuard {
	fn install(master: RawFd) -> Result<Self> {
		unsafe {
			CHILD_FD = master;
			Ok(Self(signal::signal(
				signal::SIGWINCH,
				SigHandler::Handler(sigwinch_passthrough),
			)?))
		}
	}
}

impl Drop for SigwinchGuard {
	fn drop(&mut self) { let _ = unsafe { signal::signal(signal::SIGWINCH, self.0) }; }
}

pub fn run_install(cache: Cache, config: &Config) -> Result<()> {
	// Do not run any apt scripts, Nala does this herself.
	config.apt.clear("DPkg::Pre-Invoke");
	config.apt.clear("DPkg::Post-Invoke");
	config.apt.clear("DPkg::Pre-Install-Pkgs");

	debug!("run_install");

	let (statusfd, writefd) = pipe()?;
	fcntl(&statusfd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?;

	debug!("forking");
	let window_size = unsafe { get_winsize() };
	match unsafe { forkpty(&window_size, None)? } {
		nix::pty::ForkptyResult::Child => {
			drop(statusfd);

			let child_result = (|| -> Result<()> {
				let mut progress = AcquireProgress::apt();

				let mut inst_progress = InstallProgress::fd(writefd.as_raw_fd());

				cache.commit(&mut progress, &mut inst_progress)?;
				close(writefd.into_raw_fd())?;

				stdout().flush()?;
				std::io::stderr().flush()?;
				Ok(())
			})();

			if let Err(err) = child_result {
				eprintln!("{}", t!("dpkg-child-failed", "error" => format!("{err:?}")));
				std::process::exit(1);
			}

			std::process::exit(0);
		},
		nix::pty::ForkptyResult::Parent { child, master } => {
			let mut pty = Pty::new(writefd, statusfd, master)?;

			let mut progress = Progress::new(config, true)?;
			progress.set_position(0);
			progress.set_length(100);

			while pty.listen_to_child(config, &mut progress)? {}
			check_wait_status(waitpid(child, None)?)?;

			progress.finish();
			progress.render()?;
			progress.clean_up()?;
		},
	}

	Ok(())
}

fn check_wait_status(status: WaitStatus) -> Result<()> {
	match status {
		WaitStatus::Exited(_, 0) => Ok(()),
		WaitStatus::Exited(_, code) => bail!("{}", t!("dpkg-exit", "code" => code)),
		WaitStatus::Signaled(_, signal, _) => {
			bail!("{}", t!("dpkg-exit", "code" => 128 + signal as i32))
		},
		status => bail!("Unexpected dpkg wait status: {status:?}"),
	}
}

enum PtyStr<'a> {
	Str(&'a str),
	Bytes(&'a [u8]),
	None,
	Eof,
}

pub struct Pty {
	status: File,
	_sigwinch: SigwinchGuard,
	pty: File,
	stdin: File,
	status_buf: [u8; 4096],
	status_pending: Vec<u8>,
	pty_buf: [u8; 4096],
	poll: Poll,
	events: Events,
	tokens: [(Token, Interest); 3],
	stdin_registered: bool,
}

impl fmt::Debug for Pty {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_struct("Pty")
			.field("stdin_ready", &self.stdin_ready())
			.field("pty_ready", &self.ready())
			.field("status_ready", &self.status_ready())
			.finish()
	}
}

impl Pty {
	fn new(writefd: OwnedFd, statusfd: OwnedFd, master: OwnedFd) -> Result<Pty> {
		// This is for the Parent, close the write end of the pipe.
		drop(writefd);

		let stdin = File::from(dup(std::io::stdin())?);
		let stdin_fd = stdin.as_raw_fd();
		let status_fd = statusfd.as_raw_fd();
		let master_fd = master.as_raw_fd();
		let tokens = [
			(Token(stdin_fd as usize), Interest::READABLE),
			(Token(master_fd as usize), Interest::READABLE),
			(Token(status_fd as usize), Interest::READABLE),
		];

		// Create a poll instance
		let poll = Poll::new()?;
		let events = Events::with_capacity(3);

		let stdin_registered = poll
			.registry()
			.register(&mut SourceFd(&stdin_fd), tokens[0].0, tokens[0].1)
			.is_ok();

		poll.registry()
			.register(&mut SourceFd(&master_fd), tokens[1].0, tokens[1].1)?;
		poll.registry()
			.register(&mut SourceFd(&status_fd), tokens[2].0, tokens[2].1)?;

		Ok(Pty {
			status: File::from(statusfd),
			_sigwinch: SigwinchGuard::install(master_fd)?,
			pty: File::from(master),
			stdin,
			status_buf: [0u8; 4096],
			status_pending: Vec::new(),
			pty_buf: [0u8; 4096],
			poll,
			events,
			tokens,
			stdin_registered,
		})
	}

	fn read_master(&mut self, config: &Config, progress: &mut Progress) -> Result<bool> {
		match read_fd(&mut self.pty, &mut self.pty_buf)? {
			PtyStr::Bytes(bytes) => {
				if !progress.hidden() {
					progress.hide()?;
				}

				dprog!(config, progress, "pty", "{bytes:?}");
				stdout().write_all(bytes)?;
				stdout().flush()?;
				Ok(true)
			},
			PtyStr::Str(string) => {
				if !progress.hidden()
				// Determine if it's proper to hide the progress.
				&& [SAVE_TERM, ENABLE_BRACKETED_PASTE, ENABLE_ALT_SCREEN]
					.iter()
					.any(|code| string.contains(code))
				{
					progress.hide()?;
				}

				if progress.hidden() {
					dprog!(config, progress, "pty", "{string:?}");
					write!(stdout(), "{string}")?;
					stdout().flush()?;

					if [RESTORE_TERM, DISABLE_ALT_SCREEN]
						.iter()
						.any(|code| string.contains(code))
						&& !string.contains(ENABLE_ALT_SCREEN)
					{
						progress.unhide()?;
					}

					// Don't attempt to write anything if we already wrote rawline
					return Ok(true);
				}

				for line in string.lines() {
					dprog!(config, progress, "pty", "{line:?}");

					if line.trim().is_empty() || check_spam(line) {
						continue;
					}

					// Occasionally there is a line which comes through
					if line.ends_with('\r') {
						continue;
					}

					// Sometimes just a percentage comes through "35%"
					if line.chars().nth(2).is_some_and(|c| c == '%') {
						continue;
					}

					progress.print(&msg_formatter(line))?;
				}
				Ok(true)
			},
			PtyStr::None => Ok(true),
			PtyStr::Eof => Ok(false),
		}
	}

	fn read_status(&mut self, config: &Config, progress: &mut Progress) -> Result<bool> {
		let read = match self.status.read(&mut self.status_buf) {
			Ok(0) => {
				if !self.status_pending.is_empty() {
					let line = std::mem::take(&mut self.status_pending);
					Self::process_status_line(config, progress, &line)?;
				}
				return Ok(false);
			},
			Ok(read) => read,
			Err(err) if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::Interrupted) => {
				return Ok(true);
			},
			Err(err) => return Err(err.into()),
		};

		self.status_pending
			.extend_from_slice(&self.status_buf[..read]);

		while let Some(newline) = self.status_pending.iter().position(|byte| *byte == b'\n') {
			let line = self.status_pending.drain(..=newline).collect::<Vec<_>>();
			Self::process_status_line(config, progress, &line)?;
		}

		Ok(true)
	}

	fn process_status_line(config: &Config, progress: &mut Progress, line: &[u8]) -> Result<()> {
		let Ok(line) = std::str::from_utf8(line) else {
			bail!("{}", t!("dpkg-status-utf8"));
		};
		let line = line.trim_end();
		if line.is_empty() {
			return Ok(());
		}

		let status = DpkgStatus::try_from(line)?;
		dprog!(config, progress, "statusfd", "{status:?}");

		// For ConfFile specifically, set raw
		if let DpkgStatusType::ConfFile = status.status_type {
			progress.hide()?;
		// For all other status unset raw
		} else if progress.hidden() {
			progress.unhide()?;
		}
		progress.set_position(status.percent);
		Ok(())
	}

	/// Polls Fds and checks if they're ready.
	fn poll(&mut self) -> Result<()> {
		// When resizing the terminal poll will be Error Interrupted
		// Just wait until that's not the case.
		while let Err(e) = self.poll.poll(&mut self.events, None) {
			if let ErrorKind::Interrupted = e.kind() {
				continue;
			}
			return Err(anyhow!(e));
		}

		Ok(())
	}

	fn events(&self) -> Iter<'_> { self.events.iter() }

	/// Stdin Fd is ready to be read.
	fn stdin_ready(&self) -> bool { self.stdin_registered && self.io_ready(0) }

	/// Pty master Fd is ready to be read.
	fn ready(&self) -> bool { self.io_ready(1) }

	/// Status Fd is ready to be read.
	fn status_ready(&self) -> bool { self.io_ready(2) }

	/// Helper function for the ready checkers above.
	fn io_ready(&self, i: usize) -> bool { self.events().any(|e| e.token() == self.tokens[i].0) }

	fn stdin_to_pty(&mut self) -> Result<bool> {
		let mut buffer = [0u8; 4096];
		match read_fd(&mut self.stdin, &mut buffer)? {
			PtyStr::Str(input) => {
				write!(self.pty, "{input}")?;
				Ok(true)
			},
			PtyStr::Bytes(input) => {
				self.pty.write_all(input)?;
				Ok(true)
			},
			PtyStr::None => Ok(true),
			PtyStr::Eof => Ok(false),
		}
	}

	fn listen_to_child(&mut self, config: &Config, progress: &mut Progress) -> Result<bool> {
		self.poll().context(t!("dpkg-poll"))?;

		dprog!(config, progress, "pty", "{self:?}");

		let context = t!("dpkg-read-status");
		if self.status_ready() && !self.read_status(config, progress).context(context)? {
			return Ok(false);
		}

		if self.ready() {
			return self
				.read_master(config, progress)
				.context(t!("dpkg-read-pty"));
		}

		if self.stdin_ready() {
			return self.stdin_to_pty().context(t!("dpkg-write-pty"));
		}

		Ok(true)
	}
}

fn msg_formatter(line: &str) -> String {
	let mut ret = String::new();

	let replace = [
		("Removing", t!("dpkg-removing"), Theme::Error),
		("Unpacking", t!("dpkg-unpacking"), Theme::Primary),
		("Setting up", t!("dpkg-setting-up"), Theme::Primary),
		("Processing", t!("dpkg-processing"), Theme::Primary),
	];

	for (header, change, theme) in replace {
		if !line.starts_with(header) {
			continue;
		}

		ret = line.replace(header, &color::color!(theme, &change))
	}

	if ret.ends_with("...") {
		ret = ret.replace("...", "")
	}

	if ret.is_empty() {
		return line.trim().to_string();
	}

	let regex = RegexBuilder::new(r"\(([^)]+)\)")
		.case_insensitive(true)
		.build()
		.unwrap();

	regex
		.replace_all(&ret, |caps: &regex::Captures| color::ver!(&caps[1]))
		.trim()
		.to_string()
}

fn check_spam(line: &str) -> bool {
	[
		"Nothing to fetch",
		"(Reading database",
		"Selecting previously unselected package",
		"Preparing to unpack",
	]
	.iter()
	.any(|spam| line.contains(spam))
}

fn read_fd<'a>(file: &mut File, buffer: &'a mut [u8]) -> Result<PtyStr<'a>> {
	let sized_buf = match file.read(buffer) {
		Ok(0) => return Ok(PtyStr::Eof),
		Ok(num) => &buffer[..num],
		Err(ref e) if e.kind() == ErrorKind::WouldBlock => return Ok(PtyStr::None),
		Err(ref e) if e.raw_os_error().is_some_and(|code| code == 5 || code == 4) => {
			return Ok(PtyStr::Eof);
		},
		Err(e) => return Err(anyhow!(e)),
	};

	match std::str::from_utf8(sized_buf) {
		Ok(string) => Ok(PtyStr::Str(string)),
		Err(_) => Ok(PtyStr::Bytes(sized_buf)),
	}
}

#[derive(Debug)]
enum DpkgStatusType {
	Status,
	Error,
	ConfFile,
}

#[derive(Debug)]
struct DpkgStatus {
	status_type: DpkgStatusType,
	_pkg_name: String,
	percent: u64,
	_status: String,
}

impl TryFrom<&str> for DpkgStatus {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
		let mut fields = value.splitn(4, ':');
		let status_type = match fields.next() {
			Some("pmstatus") => DpkgStatusType::Status,
			Some("pmerror") => DpkgStatusType::Error,
			Some("pmconffile") => DpkgStatusType::ConfFile,
			_ => bail!("Invalid dpkg status: {value:?}"),
		};
		let Some(pkg_name) = fields.next() else {
			bail!("Invalid dpkg status: {value:?}");
		};
		let Some(percent) = fields.next() else {
			bail!("Invalid dpkg status: {value:?}");
		};
		let Some(status) = fields.next() else {
			bail!("Invalid dpkg status: {value:?}");
		};

		Ok(DpkgStatus {
			status_type,
			_pkg_name: pkg_name.into(),
			percent: percent
				.parse::<f64>()
				.with_context(|| format!("Invalid dpkg status: {value:?}"))? as u64,
			_status: status.into(),
		})
	}
}

#[cfg(test)]
mod tests {
	use nix::sys::signal::Signal;
	use nix::unistd::Pid;

	use super::{DpkgStatus, check_wait_status};

	#[test]
	fn invalid_dpkg_status_is_an_error() {
		assert!(DpkgStatus::try_from("pmstatus:demo").is_err());
		assert!(DpkgStatus::try_from("unexpected:demo:50:working").is_err());
	}

	#[test]
	fn signaled_dpkg_is_an_error() {
		let status = nix::sys::wait::WaitStatus::Signaled(Pid::from_raw(1), Signal::SIGTERM, false);

		assert!(check_wait_status(status).is_err());
	}
}
