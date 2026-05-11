pub mod color;
pub mod configuration;
pub mod file;
pub mod keys;
pub mod logger;
pub mod paths;

use std::path::Path;

use anyhow::Result;
use clap::{ArgMatches, ColorChoice, CommandFactory, FromArgMatches};
pub use color::Theme;
pub use configuration::Config;
pub use logger::Level;
pub use paths::Paths;
use serde::{Deserialize, Serialize};

use crate::util::UnitStr;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
pub enum Switch {
	#[default]
	Auto,
	Always,
	Never,
}

impl Into<Switch> for ColorChoice {
	fn into(self) -> Switch {
		match self {
			ColorChoice::Auto => Switch::Auto,
			ColorChoice::Always => Switch::Always,
			ColorChoice::Never => Switch::Never,
		}
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum OptType {
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

impl OptType {
	pub fn as_bool(&self) -> Option<bool> {
		if let Self::Bool(value) = self {
			return Some(*value);
		}
		None
	}

	pub fn as_int(&self) -> Option<u8> {
		if let Self::Int(value) = self {
			return Some(*value);
		}
		None
	}

	pub fn as_switch(&self) -> Option<Switch> {
		if let Self::Switch(value) = self {
			return Some(*value);
		}
		None
	}

	pub fn as_string(&self) -> Option<&str> {
		if let Self::String(value) = self {
			return Some(value);
		}
		None
	}

	pub fn as_vec_string(&self) -> Option<&Vec<String>> {
		if let Self::VecString(value) = self {
			return Some(value);
		}
		None
	}
}

/// Parse CLI, resolve config path, and load configuration with fallback to
/// defaults.
pub fn bootstrap() -> Result<(ArgMatches, crate::cli::NalaParser, Config)> {
	let args = crate::cli::NalaParser::command().get_matches();
	let derived = crate::cli::NalaParser::from_arg_matches(&args)?;

	let config_path = derived
		.config
		.as_deref()
		.unwrap_or(Path::new("/etc/nala/nala.conf"));

	let config = match Config::new(config_path) {
		Ok(config) => config,
		Err(err) => {
			// If user explicitly asked for a config file, bubble the error.
			if derived.config.is_some() {
				return Err(err);
			}
			Config::default()
		},
	};

	let mut config = config;

	if let Ok(env_dir) = std::env::var("NALA_HISTORY_DIR") {
		config.set_history_dir(env_dir);
	}

	if let Some(dir) = derived.history_dir.as_deref() {
		config.set_history_dir(dir.to_string_lossy().to_string());
	}

	Ok((args, derived, config))
}
