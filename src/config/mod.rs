pub mod color;
pub mod configuration;
#[macro_use]
pub mod logger;
pub mod paths;

pub use color::Theme;
pub use configuration::Config;
pub use paths::Paths;
use serde::{Deserialize, Serialize};

use crate::tui::UnitStr;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum Switch {
	Always,
	Never,
	Auto,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum Opt {
	Bool(bool),
	Int(u8),
	Int64(u64),
	Switch(Switch),
	UnitStr(UnitStr),
	// Strings have to be last in the enum
	// as almost anything will match them
	String(String),
	VecString(Vec<String>),
}

impl From<&bool> for Opt {
	fn from(value: &bool) -> Self { Opt::Bool(*value) }
}

impl From<&u8> for Opt {
	fn from(value: &u8) -> Self { Opt::Int(*value) }
}

impl From<&u64> for Opt {
	fn from(value: &u64) -> Self { Opt::Int64(*value) }
}

impl FromIterator<String> for Opt {
	fn from_iter<I: IntoIterator<Item = String>>(iter: I) -> Self {
		Opt::VecString(iter.into_iter().collect())
	}
}

#[macro_export]
macro_rules! error {
	($($arg:tt)+) => (log::log!(log::Level::Error, $($arg)+))
}

#[macro_export]
macro_rules! warning {
	($($arg:tt)+) => (log::log!(log::Level::Warn, $($arg)+))
}

#[macro_export]
macro_rules! notice {
	($($arg:tt)+) => (log::log!(log::Level::Info, $($arg)+))
}

#[macro_export]
macro_rules! verbose {
	($($arg:tt)+) => (log::log!(log::Level::Debug, $($arg)+))
}

#[macro_export]
macro_rules! debug {
	($($arg:tt)+) => (log::log!(log::Level::Trace, $($arg)+))
}

#[macro_export]
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
