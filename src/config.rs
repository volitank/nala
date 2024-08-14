use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use crossterm::tty::IsTty;
use rust_apt::config::Config as AptConfig;
use serde::{Deserialize, Serialize};

use crate::colors::{RatStyle, Style, Theme};
use crate::tui::progress::{NumSys, UnitStr};

/// Represents different file and directory paths
pub enum Paths {
	/// The Archive dir holds packages.
	/// Default dir `/var/cache/apt/archives/`
	Archive,
	/// The Lists dir hold package lists from `update` command.
	/// Default dir `/var/lib/apt/lists/`
	Lists,
	/// The main Source List.
	/// Default file `/etc/apt/sources.list`
	SourceList,
	/// The Sources parts directory
	/// Default dir `/etc/apt/sources.list.d/`
	SourceParts,
	/// Nala Sources file is generated from the `fetch` command.
	/// Default file `/etc/apt/sources.list.d/nala-sources.list`
	NalaSources,

	History,
}

impl Paths {
	pub fn path(&self) -> &'static str {
		match self {
			Paths::Archive => "Dir::Cache::Archives",
			Paths::Lists => "Dir::State::Lists",
			Paths::SourceList => "Dir::Etc::sourcelist",
			Paths::SourceParts => "Dir::Etc::sourceparts",
			Paths::NalaSources => "/etc/apt/sources.list.d/nala.sources",
			Paths::History => "/var/lib/nala/history",
		}
	}

	pub fn default_path(&self) -> &'static str {
		match self {
			Paths::Archive => "/var/cache/apt/archives/",
			Paths::Lists => "/var/lib/apt/lists/",
			Paths::SourceList => "/etc/apt/sources.list",
			Paths::SourceParts => "/etc/apt/sources.list.d/",
			Paths::NalaSources => self.path(),
			Paths::History => self.path(),
		}
	}
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Switch {
	Always,
	Never,
	Auto,
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

#[derive(Serialize, Deserialize, Debug)]
/// Configuration struct
pub struct Config {
	#[serde(rename(deserialize = "Nala"), default)]
	map: HashMap<String, OptType>,

	#[serde(rename(deserialize = "Theme"), default)]
	theme: HashMap<Theme, Style>,

	// The following fields are not used with serde
	#[serde(skip)]
	pub apt: AptConfig,

	#[serde(skip)]
	/// The command that is being run
	pub command: String,
}

impl Default for Config {
	/// The default configuration for Nala.
	fn default() -> Config {
		let mut config = Config {
			map: HashMap::new(),
			theme: HashMap::new(),
			apt: AptConfig::new(),
			command: "Command Not Given Yet".to_string(),
		};
		config.set_default_theme();
		config
	}
}

impl Config {
	pub fn new(conf_file: &Path) -> Result<Config> {
		// Try to read the entire config file and map it.
		// Return an empty config and print a warning on failure.
		let mut map = Self::read_config(conf_file)?;
		map.set_default_theme();
		Ok(map)
	}

	pub fn set_default_theme(&mut self) {
		for theme in [
			Theme::Primary,
			Theme::Secondary,
			Theme::Regular,
			Theme::Highlight,
			Theme::ProgressFilled,
			Theme::ProgressUnfilled,
			Theme::Notice,
			Theme::Warning,
			Theme::Error,
		] {
			if self.theme.contains_key(&theme) {
				continue;
			}
			self.theme.insert(theme, theme.default_style());
		}
	}

	pub fn rat_style(&self, theme: Theme) -> RatStyle {
		self.theme.get(&theme).unwrap_or(&Style::default()).to_rat()
	}

	pub fn can_color(&self) -> bool {
		if let Some(OptType::Switch(switch)) = self.map.get("color") {
			match switch {
				Switch::Always => return true,
				Switch::Never => return false,
				Switch::Auto => return std::io::stdout().is_tty(),
			}
		}
		false
	}

	pub fn color(&self, theme: Theme, string: &str) -> String {
		if self.can_color() {
			if let Some(theme) = self.theme.get(&theme) {
				return format!("{theme}{string}\x1b[0m");
			}
		}
		string.to_string()
	}

	/// Hightlights the string according to configuration.
	pub fn highlight(&self, string: &str) -> String { self.color(Theme::Highlight, string) }

	/// Color the version according to configuration.
	pub fn color_ver(&self, string: &str) -> String {
		format!(
			"{}{}{}",
			self.highlight("("),
			self.color(Theme::Secondary, string),
			self.highlight(")")
		)
	}

	/// Print a notice to stderr
	pub fn stderr(&self, theme: Theme, string: &str) {
		let header = match theme {
			Theme::Error => "Error:",
			Theme::Warning => "Warning:",
			Theme::Notice => "Notice:",
			_ => panic!("'{theme:?}' is not a valid stderr!"),
		};
		eprintln!("{} {string}", self.color(theme, header));
	}

	/// Read and Return the entire toml configuration file
	fn read_config(conf_file: &Path) -> Result<Config> {
		let conf = fs::read_to_string(conf_file)
			.with_context(|| format!("Failed to read {}, using defaults", conf_file.display()))?;

		let config: Config = toml::from_str(&conf)
			.with_context(|| format!("Failed to parse {}, using defaults", conf_file.display()))?;

		Ok(config)
	}

	/// Load configuration with the command line arguments
	pub fn load_args(&mut self, args: &ArgMatches) {
		for id in args.ids() {
			let key = id.as_str().to_string();
			// Don't do anything if the option wasn't specifically passed
			if Some(ValueSource::CommandLine) != args.value_source(&key) {
				continue;
			}

			if let Ok(Some(value)) = args.try_get_one::<bool>(&key) {
				self.map.insert(key, OptType::Bool(*value));
				continue;
			}

			if let Ok(Some(value)) = args.try_get_occurrences::<String>(&key) {
				self.map
					.insert(key, OptType::VecString(value.flatten().cloned().collect()));
				continue;
			}

			if let Ok(Some(value)) = args.try_get_one::<u8>(&key) {
				self.map.insert(key, OptType::Int(*value));
				continue;
			}

			if let Ok(Some(value)) = args.try_get_one::<u64>(&key) {
				self.map.insert(key, OptType::Int64(*value));
			}
		}

		// Set the color option if it doesn't exist
		if !self.map.contains_key("color") {
			self.map
				.insert("color".to_string(), OptType::Switch(Switch::Auto));
		}

		// If Debug is there we can print the whole thing.
		if self.debug() {
			dbg!(&self);
		}
	}

	/// Get a bool from the configuration.
	pub fn get_bool(&self, key: &str, default: bool) -> bool {
		if let Some(OptType::Bool(bool)) = self.map.get(key) {
			return *bool;
		}
		default
	}

	/// Set a bool in the configuration.
	pub fn set_bool(&mut self, key: &str, value: bool) {
		self.map.insert(key.to_string(), OptType::Bool(value));
	}

	/// Get a single str from the configuration.
	pub fn get_str(&self, key: &str) -> Option<&str> {
		if let OptType::VecString(vec) = self.map.get(key)? {
			return vec.first().map(|x| x.as_str());
		}

		if let OptType::String(str) = self.map.get(key)? {
			return Some(str);
		}
		None
	}

	/// Get a Vec of Strings from the configuration.
	pub fn get_vec(&self, key: &str) -> Option<&Vec<String>> {
		if let OptType::VecString(vec) = self.map.get(key)? {
			return Some(vec);
		}
		None
	}

	/// Get a file from the configuration based on the Path enum.
	pub fn get_file(&self, file: &Paths) -> String {
		match file {
			// For now NalaSources is hard coded.
			Paths::NalaSources => file.path().to_string(),
			Paths::History => file.path().to_string(),
			_ => self.apt.file(file.path(), file.default_path()),
		}
	}

	/// Get a path from the configuration based on the Path enum.
	pub fn get_path(&self, dir: &Paths) -> PathBuf {
		PathBuf::from(match dir {
			// For now NalaSources is hard coded.
			Paths::NalaSources => dir.path().to_string(),
			Paths::History => dir.path().to_string(),
			// Everything else should be an Apt Path
			_ => self.apt.file(dir.path(), dir.default_path()),
		})
	}

	// TODO: Combine these into a function

	/// Should the TUI be shown?
	pub fn show_tui(&self) -> bool {
		if self.get_bool("no_tui", false) {
			return false;
		}
		self.get_bool("tui", true)
	}

	pub fn full_upgrade(&self) -> bool {
		if self.get_bool("no_full", false) {
			return false;
		}
		self.get_bool("full", true)
	}

	/// Get the package names that were passed as arguments.
	pub fn pkg_names(&self) -> Option<&Vec<String>> { self.get_vec("pkg_names") }

	/// Get the countries that were passed as arguments.
	pub fn countries(&self) -> Option<&Vec<String>> { self.get_vec("countries") }

	/// If fetch should be in auto mode and how many mirrors to get.
	pub fn auto(&self) -> Option<u8> {
		if let OptType::Int(value) = self.map.get("auto")? {
			return Some(*value);
		}
		None
	}

	pub fn unit_str(&self, unit: u64) -> String {
		if let Some(OptType::UnitStr(value)) = self.map.get("UnitStr") {
			return value.str(unit);
		}
		UnitStr::new(0, NumSys::Binary).str(unit)
	}

	/// Return true if debug is enabled
	pub fn debug(&self) -> bool { self.get_bool("debug", false) }

	/// Return true if verbose or debug is enabled
	pub fn verbose(&self) -> bool { self.get_bool("verbose", self.debug()) }
}

#[cfg(test)]
mod test {
	use crate::tui::progress::{NumSys, UnitStr};
	use crate::Config;

	#[test]
	fn serialize_config() {
		let mut config = Config::default();
		config.set_default_theme();

		config.map.insert(
			"unit_str".to_string(),
			super::OptType::UnitStr(UnitStr::new(0, NumSys::Binary)),
		);

		let toml = toml::to_string_pretty(&config).unwrap();

		println!("{toml}")
	}
}
