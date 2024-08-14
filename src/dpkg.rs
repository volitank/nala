use std::fmt;
use std::fs::File;
use std::io::{stdout, ErrorKind, Read, Write};
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd, RawFd};

use anyhow::{anyhow, bail, Context, Result};
use mio::event::Iter;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Token};
use nix::fcntl::{fcntl, FcntlArg, OFlag};
use nix::libc::{winsize, TIOCGWINSZ, TIOCSWINSZ};
use nix::pty::forkpty;
use nix::sys::signal::{self, SigHandler};
use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{close, pipe, Pid};
use nix::{ioctl_read_bad, ioctl_write_ptr_bad};
use regex::RegexBuilder;
use rust_apt::progress::{AcquireProgress, InstallProgress};
use rust_apt::Cache;

use crate::colors::Theme;
use crate::config::Config;
use crate::tui::NalaProgressBar;
use crate::{dprint, dprog};

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
// const DISABLE_ALT_SCREEN: &str = "\x1b[?1049l";
// const SHOW_CURSOR: &'static str = "\x1b[?25h";
// const HIDE_CURSOR: &'static str = "\x1b[?25l";
// const SET_CURSER: &'static str = "\x1b[?1l";
const SAVE_TERM: &str = "\x1b[22;0;0t";
// const RESTORE_TERM: &str = "\x1b[23;0;0t";
// const APPLICATION_KEYPAD: &'static str = "\x1b=";
// const NORMAL_KEYPAD: &'static str = "\x1b>";
// const CR: &'static str = "\r";
// const LF: &'static str = "\n";
// const CRLF: &'static str = "\r\n";

static mut CHILD_FD: i32 = 0;
const STDIN_FD: i32 = 0;
const STDOUT_FD: i32 = 1;
const STDERR_FD: i32 = 2;

// Define the ioctl read call for TIOCGWINSZ
ioctl_read_bad!(tiocgwinsz, TIOCGWINSZ, winsize);
// Define the ioctl write call for TIOCSWINSZ
ioctl_write_ptr_bad!(tiocswinsz, TIOCSWINSZ, winsize);

/// Get Terminal Size from stdin
unsafe fn get_winsize() -> nix::Result<winsize> {
	let mut ws: winsize = unsafe { zeroed() };
	tiocgwinsz(STDIN_FD, &mut ws)?;
	Ok(ws)
}

extern "C" fn sigwinch_passthrough(_: i32) {
	unsafe {
		// Get Terminal Size from stdin.
		let ws = get_winsize().unwrap();
		// Set Terminal Size for pty.
		tiocswinsz(CHILD_FD, &ws).unwrap();
	}
}

pub fn run_install(cache: Cache, config: &Config) -> Result<()> {
	// Do not run any apt scripts, Nala does this herself.
	config.apt.clear("DPkg::Pre-Invoke");
	config.apt.clear("DPkg::Post-Invoke");
	config.apt.clear("DPkg::Pre-Install-Pkgs");

	dprint!(config, "run_install");

	let (statusfd, writefd) = pipe()?;
	fcntl(statusfd.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK))?;

	dprint!(config, "forking");
	let window_size = unsafe { get_winsize()? };
	match unsafe { forkpty(&window_size, None)? } {
		nix::pty::ForkptyResult::Child => {
			close(statusfd.as_raw_fd())?;

			let mut progress = AcquireProgress::apt();

			let mut inst_progress = InstallProgress::fd(writefd.as_raw_fd());

			cache.commit(&mut progress, &mut inst_progress)?;
			close(writefd.into_raw_fd())?;

			// Flush all stdio for the child before we leave.
			for fd in [STDIN_FD, STDOUT_FD, STDERR_FD] {
				let mut file = unsafe { File::from_raw_fd(fd) };
				file.flush()?;
			}
		},
		nix::pty::ForkptyResult::Parent { child, master } => {
			let mut pty = Pty::new(
				writefd.into_raw_fd(),
				statusfd.as_raw_fd(),
				master.as_raw_fd(),
			)?;

			let mut progress = NalaProgressBar::new(config, true)?;
			progress.indicatif.set_position(0);
			progress.indicatif.set_length(100);

			while pty.listen_to_child(config, &mut progress, child)? {}

			progress.indicatif.finish();
			progress.render()?;
			progress.clean_up()?;
		},
	}

	Ok(())
}

enum PtyStr<'a> {
	Str(&'a str),
	None,
	Eof,
}

pub struct Pty {
	status: File,
	pty: File,
	stdin: File,
	status_buf: [u8; 4096],
	pty_buf: [u8; 4096],
	poll: Poll,
	events: Events,
	tokens: [(Token, Interest); 3],
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
	fn new(writefd: RawFd, statusfd: RawFd, master: RawFd) -> Result<Pty> {
		// This is for the Parent, close the write end of the pipe.
		close(writefd)?;

		let tokens = [
			(Token(0), Interest::READABLE),
			(Token(master as usize), Interest::READABLE),
			(Token(statusfd as usize), Interest::READABLE),
		];

		// Create a poll instance
		let poll = Poll::new()?;
		let events = Events::with_capacity(3);

		for token in tokens {
			poll.registry()
				.register(&mut SourceFd(&(token.0 .0 as i32)), token.0, token.1)?;
		}

		unsafe {
			CHILD_FD = master;
			signal::signal(signal::SIGWINCH, SigHandler::Handler(sigwinch_passthrough))?;

			Ok(Pty {
				status: File::from_raw_fd(statusfd),
				pty: File::from_raw_fd(master),
				stdin: File::from_raw_fd(0),
				status_buf: [0u8; 4096],
				pty_buf: [0u8; 4096],
				poll,
				events,
				tokens,
			})
		}
	}

	fn read_master(&mut self, config: &Config, progress: &mut NalaProgressBar) -> Result<bool> {
		match read_fd(&mut self.pty, &mut self.pty_buf)? {
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
					// TODO: We may not need the following code, but I don't want to get rid of it
					// just yet At least until we test specifically Dialog

					// // After writing the line to the terminal check is we can leave raw mode.
					// if (string.contains(RESTORE_TERM) | string.contains(DISABLE_ALT_SCREEN))
					// 	// Fix for Dialog Debconf Frontend https://gitlab.com/volian/nala/-/issues/211
					// 	&& !string.contains(ENABLE_ALT_SCREEN)
					// {
					// 	progress.unset_raw()?;
					// 	self.raw = false;
					// }

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

					progress.print(&msg_formatter(config, line))?;
				}
				Ok(true)
			},
			PtyStr::None => Ok(true),
			PtyStr::Eof => Ok(false),
		}
	}

	fn read_status(&mut self, config: &Config, progress: &mut NalaProgressBar) -> Result<bool> {
		match read_fd(&mut self.status, &mut self.status_buf)? {
			PtyStr::Str(string) => {
				for line in string.lines() {
					let status = DpkgStatus::try_from(line)?;
					dprog!(config, progress, "statusfd", "{status:?}");

					// For ConfFile specifically, set raw
					if let DpkgStatusType::ConfFile = status.status_type {
						progress.hide()?;
					// For all other status unset raw
					} else if progress.hidden() {
						progress.unhide()?;
					}
					progress.indicatif.set_position(status.percent);
				}
				Ok(true)
			},
			PtyStr::None => Ok(true),
			PtyStr::Eof => Ok(false),
		}
	}

	/// Checks the status of the child, polls Fds and checks if they're ready.
	fn poll(&mut self, child: Pid) -> Result<bool> {
		// Wait for the child process to finish and get its exit code
		let wait_status = waitpid(child, Some(WaitPidFlag::WNOHANG))?;
		if let WaitStatus::Exited(_, exit_code) = wait_status {
			if exit_code != 0 {
				bail!("Dpkg exited with code: '{exit_code}'");
			}
		}

		// When resizing the terminal poll will be Error Interrupted
		// Just wait until that's not the case.
		while let Err(e) = self.poll.poll(&mut self.events, None) {
			if let ErrorKind::Interrupted = e.kind() {
				continue;
			}
			return Err(anyhow!(e));
		}

		Ok(!self.is_read_closed())
	}

	fn is_read_closed(&self) -> bool {
		self.events
			.iter()
			.any(|e| e.token() == self.tokens[1].0 && e.is_read_closed())
	}

	fn events(&self) -> Iter<'_> { self.events.iter() }

	/// Stdin Fd is ready to be read.
	fn stdin_ready(&self) -> bool { self.io_ready(0) }

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
			PtyStr::None => Ok(true),
			PtyStr::Eof => Ok(false),
		}
	}

	fn listen_to_child(
		&mut self,
		config: &Config,
		progress: &mut NalaProgressBar,
		child: Pid,
	) -> Result<bool> {
		if !self.poll(child).context("Unable to poll child")? {
			return Ok(false);
		}

		dprog!(config, progress, "pty", "{self:?}");

		let context = "Unable to read Status Fd";
		if self.status_ready() && !self.read_status(config, progress).context(context)? {
			return Ok(false);
		}

		if self.ready() {
			return self
				.read_master(config, progress)
				.context("Unable to read from pty");
		}

		if self.stdin_ready() {
			return self.stdin_to_pty().context("Unable to send stdin to pty");
		}

		Ok(true)
	}
}

fn msg_formatter(config: &Config, line: &str) -> String {
	let mut ret = String::new();

	let replace = [
		("Removing", "Removing:", Theme::Error),
		("Unpacking", "Unpacking:", Theme::Primary),
		("Setting up", "Setting up:", Theme::Primary),
		("Processing", "Processing:", Theme::Primary),
	];

	for (header, change, theme) in replace {
		if !line.starts_with(header) {
			continue;
		}

		ret = line.replace(header, &config.color(theme, change))
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
		.replace_all(&ret, |caps: &regex::Captures| {
			let version_string = &caps[1];
			config.color_ver(version_string)
		})
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
			return Ok(PtyStr::Eof)
		},
		Err(e) => return Err(anyhow!(e)),
	};

	Ok(PtyStr::Str(std::str::from_utf8(sized_buf)?))
}

#[derive(Debug, Default)]
enum DpkgStatusType {
	Status,
	Error,
	ConfFile,
	#[default]
	None,
}

impl From<&str> for DpkgStatusType {
	fn from(value: &str) -> Self {
		match value {
			"pmstatus" => Self::Status,
			"pmerror" => Self::Error,
			"pmconffile" => Self::ConfFile,
			_ => unreachable!(),
		}
	}
}

#[derive(Debug, Default)]
struct DpkgStatus {
	status_type: DpkgStatusType,
	_pkg_name: String,
	percent: u64,
	_status: String,
}

impl TryFrom<&str> for DpkgStatus {
	type Error = anyhow::Error;

	fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
		let status: Vec<&str> = value.split(':').collect();

		Ok(DpkgStatus {
			status_type: DpkgStatusType::from(status[0]),
			_pkg_name: status[1].into(),
			percent: status[2].parse::<f64>()? as u64,
			_status: status[3].into(),
		})
	}
}
