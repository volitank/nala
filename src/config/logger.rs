use std::io::Write;
use std::sync::Mutex;

use crate::config::color::Target;
use crate::config::{color, Theme};

static LOG: std::sync::LazyLock<Mutex<Logger>> =
	std::sync::LazyLock::new(|| Mutex::new(Logger::new(LogOptions::default())));

#[cfg(test)]
pub fn setup_logger(options: LogOptions) -> &'static Mutex<Logger> {
	let mut logger = LOG.lock().unwrap();
	*logger = Logger::new(options);
	drop(logger);
	&LOG
}

pub fn get_logger() -> &'static Mutex<Logger> { &LOG }

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
			color::color_str_with_target(level.as_theme(), level.as_str(), Target::Stderr)
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
	use std::sync::{LazyLock, Mutex, MutexGuard};

	use nix::fcntl::{fcntl, FcntlArg, OFlag};

	use super::Level;
	use crate::config::color::{setup_color, Color};
	use crate::config::logger::*;

	static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

	fn test_lock() -> MutexGuard<'static, ()> { TEST_LOCK.lock().unwrap() }

	struct LoggerTest {
		reader: File,
		_guard: MutexGuard<'static, ()>,
	}

	impl LoggerTest {
		fn new(level: Level) -> Self {
			let guard = test_lock();
			let (statusfd, writefd) = nix::unistd::pipe().unwrap();
			// This way it will error if the io is blocked
			fcntl(&statusfd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

			let writer = File::from(writefd);
			let reader = File::from(statusfd);

			setup_color(Color::new(crate::config::Switch::Never, Default::default()));
			setup_logger(LogOptions::new(level, Box::new(writer)));
			Self {
				reader,
				_guard: guard,
			}
		}

		fn read_exact(&mut self, size: usize) -> std::io::Result<Vec<u8>> {
			let mut output = vec![0; size];
			self.reader.read_exact(&mut output)?;
			Ok(output)
		}
	}

	impl Drop for LoggerTest {
		fn drop(&mut self) {
			setup_logger(LogOptions::default());
			setup_color(Color::default());
		}
	}

	#[test]
	fn info() {
		let mut logger = LoggerTest::new(Level::Info);

		info!("Test");

		let expected = "Notice: Test\n";
		let output = logger.read_exact(expected.len()).unwrap();

		assert_eq!(std::str::from_utf8(&output).unwrap(), expected);

		// Test that debug does not work
		debug!("Test");
		assert!(logger.read_exact("Debug: Test\n".len()).is_err());
	}

	#[test]
	fn debug() {
		let mut logger = LoggerTest::new(Level::Debug);

		debug!("Test");
		let debug_expected = "Debug: Test\n";
		let output = logger.read_exact(debug_expected.len()).unwrap();
		assert_eq!(std::str::from_utf8(&output).unwrap(), debug_expected);

		// Test that info during debug does work
		info!("Test");
		let notice_expected = "Notice: Test\n";
		let output = logger.read_exact(notice_expected.len()).unwrap();
		assert_eq!(std::str::from_utf8(&output).unwrap(), notice_expected);
	}
}
