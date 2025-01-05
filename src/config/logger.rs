use std::io::Write;
// use tokio::sync::Mutex;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::config::{color, Theme};

static LOG: OnceLock<Mutex<Logger>> = OnceLock::new();

pub fn setup_logger(options: LogOptions) -> &'static Mutex<Logger> {
	LOG.get_or_init(|| Mutex::new(Logger::new(options)))
}

pub fn get_logger() -> &'static Mutex<Logger> { LOG.get().unwrap() }

#[macro_export]
macro_rules! log {
	($level:path, $($arg: tt)*) => {{
		let string = std::fmt::format(std::format_args!($($arg)*));
		$crate::config::logger::get_logger()
			.lock()
			.unwrap()
			.log($level, &string);
	}};
}

#[macro_export]
/// Print Debug information if the option is set
macro_rules! debug {
	($($arg: tt)*) => {{
		$crate::log!($crate::config::Level::Debug, $($arg)*)
	}};
}

#[macro_export]
/// Print Debug information if the option is set
macro_rules! info {
	($($arg: tt)*) => {{
		$crate::log!($crate::config::Level::Notice, $($arg)*)
	}};
}

#[macro_export]
macro_rules! warn {
	($($arg: tt)*) => {{
		$crate::log!($crate::config::Level::Warning, $($arg)*)
	}};
}

#[macro_export]
macro_rules! error {
	($($arg: tt)*) => {{
		$crate::log!($crate::config::Level::Error, $($arg)*)
	}};
}

type LogWriter = Box<dyn Write + Send + Sync>;

pub struct LogOptions {
	level: Level,
	out: LogWriter,
}

impl LogOptions {
	pub fn new(level: Level, out: LogWriter) -> LogOptions { Self { level, out } }
}

impl std::fmt::Debug for LogOptions {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("LogOptions")
			.field("level", &self.level)
			.finish()
	}
}

impl Default for LogOptions {
	fn default() -> Self { Self::new(Level::Info, Box::new(std::io::stderr())) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level {
	Error,
	Notice,
	Warning,
	Info,
	Verbose,
	Debug,
}

impl Level {
	pub fn as_str(&self) -> &str { self.as_ref() }

	pub fn as_theme(&self) -> &Theme { self.as_ref() }
}

impl AsRef<str> for Level {
	fn as_ref(&self) -> &str {
		match self {
			Self::Error => "Error:",
			Self::Notice => "Notice:",
			Self::Warning => "Warning:",
			Self::Info => "Info:",
			Self::Verbose => "Verbose:",
			Self::Debug => "Debug:",
		}
	}
}

impl AsRef<Theme> for Level {
	fn as_ref(&self) -> &Theme {
		match self {
			Self::Error => &Theme::Error,
			Self::Notice => &Theme::Notice,
			Self::Warning => &Theme::Warning,
			Self::Info => &Theme::Highlight,
			Self::Verbose => &Theme::Highlight,
			Self::Debug => &Theme::Highlight,
		}
	}
}

#[derive(Debug)]
pub struct Logger(LogOptions);

impl Logger {
	pub fn new(options: LogOptions) -> Logger { Logger(options) }

	pub fn should_log(&self, msg_level: Level) -> bool {
		match msg_level {
			// Always display Error, Notice, Warning, Info,
			// The only real log levels are Info, Verbose, Debug
			Level::Error | Level::Notice | Level::Warning | Level::Info => true,
			Level::Verbose => matches!(self.level(), Level::Verbose | Level::Debug),
			Level::Debug => matches!(self.level(), Level::Debug),
		}
	}

	pub fn log(&mut self, level: Level, msg: &str) {
		if !self.should_log(level) {
			return;
		}

		writeln!(
			self.0.out,
			"{} {msg}",
			color::color!(level.as_theme(), level.as_str())
		)
		.unwrap();
	}

	pub fn level(&self) -> Level { self.0.level }

	pub fn set_level(&mut self, level: Level) { self.0.level = level; }
}

#[cfg(test)]
mod tests {
	use std::fs::File;
	use std::io::Read;
	use std::os::fd::AsRawFd;

	use nix::fcntl::{fcntl, FcntlArg, OFlag};

	use super::Level;
	use crate::config::logger::*;

	fn read_write() -> (File, File) {
		let (statusfd, writefd) = nix::unistd::pipe().unwrap();
		// This way it will error if the io is blocked
		fcntl(statusfd.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

		let writer = File::from(writefd);
		let reader = File::from(statusfd);

		(reader, writer)
	}

	fn read_exact(reader: &mut File, size: usize) -> std::io::Result<Vec<u8>> {
		let mut v = vec![0; size];
		reader.read_exact(&mut v)?;
		Ok(v)
	}

	#[test]
	fn info() {
		let (mut reader, writer) = read_write();
		setup_logger(LogOptions::new(Level::Info, Box::new(writer)));

		info!("Test");

		let output = read_exact(&mut reader, 11).unwrap();

		assert_eq!(std::str::from_utf8(&output).unwrap(), "Info: Test\n");

		// Test that debug does not work
		debug!("Test");
		assert!(read_exact(&mut reader, 11).is_err());
	}

	#[test]
	fn debug() {
		let (mut reader, writer) = read_write();
		setup_logger(LogOptions::new(Level::Debug, Box::new(writer)));

		debug!("Test");
		let output = read_exact(&mut reader, 12).unwrap();
		assert_eq!(std::str::from_utf8(&output).unwrap(), "Debug: Test\n");

		// Test that info during debug does work
		info!("Test");
		let output = read_exact(&mut reader, 11).unwrap();
		assert_eq!(std::str::from_utf8(&output).unwrap(), "Info: Test\n");
	}
}
