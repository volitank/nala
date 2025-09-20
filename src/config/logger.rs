use std::fmt::Debug;
use std::io::{Stderr, Write};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard};

use anyhow::Result;
use log::LevelFilter;
use serde::{Deserialize, Serialize};

use super::color::Color;
use crate::config::{color, Theme};

pub const DISABLED: AtomicBool = AtomicBool::new(false);

#[macro_export]
/// Print Debug information using NalaProgress.
macro_rules! dprog {
	($config:expr, $term:expr, $progress:expr, $context:expr, $(,)? $($arg:tt)*) => {
		if $config.debug() {
			let output = std::fmt::format(std::format_args!($($arg)*));
			if $progress.hidden() {
				eprintln!("DEBUG({}): {output}", $context);
			} else {
				$progress.print($config, $term, &format!("DEBUG({}): {output}", $context))?;
			}
		}
	};
}

macro_rules! rip {
	($result:expr) => {
		match $result {
			Ok(ok) => ok,
			Err(err) => {
				eprintln!("\x1b[1;91mError:\x1b[0m {err:?}");
			},
		}
	};
}

pub trait LogWriter: Write + Debug + Send + Sync + 'static {}
impl LogWriter for Stderr {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Level {
	Error,
	Warn,
	Notice,
	Verbose,
	Debug,
}

impl From<log::Level> for Level {
	fn from(value: log::Level) -> Self {
		match value {
			log::Level::Error => Level::Error,
			log::Level::Warn => Level::Warn,
			log::Level::Info => Level::Notice,
			log::Level::Debug => Level::Verbose,
			log::Level::Trace => Level::Debug,
		}
	}
}

impl AsRef<str> for Level {
	fn as_ref(&self) -> &str {
		match self {
			Self::Error => "Error:",
			Self::Notice => "Notice:",
			Self::Warn => "Warning:",
			Self::Verbose => "Verbose:",
			Self::Debug => "Debug:",
		}
	}
}

impl From<Level> for Theme {
	fn from(value: Level) -> Self {
		match value {
			Level::Error => Theme::Error,
			Level::Warn => Theme::Warning,
			Level::Notice => Theme::Notice,
			Level::Verbose => Theme::Highlight,
			Level::Debug => Theme::Highlight,
		}
	}
}

pub struct Writer(Mutex<Box<dyn LogWriter>>);

impl Writer {
	fn new<T: LogWriter>(writer: T) -> Writer { Self(Mutex::new(Box::new(writer))) }

	fn writer(&self) -> MutexGuard<'_, Box<(dyn LogWriter)>> {
		match self.0.lock() {
			Ok(ok) => ok,
			Err(err) => {
				eprintln!("\x1b[1;91mError:\x1b[0m {err:?}");
				err.into_inner()
			},
		}
	}
}

impl Default for Writer {
	fn default() -> Self { Self::new(std::io::stderr()) }
}

#[derive(Default)]
pub struct Logger {
	opts: Color,
	out: Writer,
}

impl Logger {
	pub fn new(opts: Color, out: Writer) -> Logger { Self { opts, out } }

	pub fn init(self) -> Result<()> {
		log::set_max_level(LevelFilter::max());
		log::set_boxed_logger(Box::new(self))?;
		Ok(())
	}
}

impl log::Log for Logger {
	fn enabled(&self, metadata: &log::Metadata) -> bool {
		if DISABLED.into_inner() {
			return false;
		}
		match Level::from(metadata.level()) {
			// Always display Error, Notice, Warning, Info,
			// The only real log levels are Info, Verbose, Debug
			Level::Error | Level::Warn | Level::Notice => true,
			Level::Verbose => matches!(self.opts.level, Level::Verbose | Level::Debug),
			Level::Debug => matches!(self.opts.level, Level::Debug),
		}
	}

	fn log(&self, record: &log::Record) {
		if self.enabled(record.metadata()) {
			let level = Level::from(record.level());

			rip!(writeln!(
				self.out.writer(),
				"{} {}",
				color::color!(level.into(), level),
				record.args()
			));
		}
	}

	// No errors in here, just throw it away flush don't matter
	fn flush(&self) {
		rip!(self.out.writer().flush());
	}
}

#[cfg(test)]
mod tests {
	// 	use std::fs::File;
	// 	use std::io::Read;
	// 	use std::os::fd::AsRawFd;

	// 	use logger::Logger;
	// 	use nix::fcntl::{fcntl, FcntlArg, OFlag};

	// 	use crate::config::*;

	// 	fn read_write() -> (File, File) {
	// 		let (statusfd, writefd) = nix::unistd::pipe().unwrap();
	// 		// This way it will error if the io is blocked
	// 		fcntl(statusfd.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

	// 		let writer = File::from(writefd);
	// 		let reader = File::from(statusfd);

	// 		(reader, writer)
	// 	}

	// 	fn read_exact(reader: &mut File, size: usize) -> std::io::Result<Vec<u8>> {
	// 		let mut v = vec![0; size];
	// 		reader.read_exact(&mut v)?;
	// 		Ok(v)
	// 	}

	// #[test]
	// fn info() {
	// 	let (mut reader, writer) = read_write();
	// 	let _ = Logger::default().init().unwrap();

	// 	crate::notice!("Test");

	// 	let output = read_exact(&mut reader, 11).unwrap();

	// 	assert_eq!(std::str::from_utf8(&output).unwrap(), "Info: Test\n");

	// 	// Test that debug does not work
	// 	crate::debug!("Test");
	// 	assert!(read_exact(&mut reader, 11).is_err());
	// }

	// #[test]
	// fn debug() {
	// 	let (mut reader, writer) = read_write();
	// 	Logger::default().init();

	// 	crate::debug!("Test");
	// 	let output = read_exact(&mut reader, 12).unwrap();
	// 	assert_eq!(std::str::from_utf8(&output).unwrap(), "Debug: Test\n");

	// 	// Test that info during debug does work
	// 	crate::notice!("Test");
	// 	let output = read_exact(&mut reader, 11).unwrap();
	// 	assert_eq!(std::str::from_utf8(&output).unwrap(), "Info: Test\n");
	// }

	#[test]
	fn serialize() {
		let color = crate::config::color::Color::default();
		let json = serde_json::to_string_pretty(&color).unwrap();
		println!("{json}");
	}
}
