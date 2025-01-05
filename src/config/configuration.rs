use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use ratatui::style::Styled;
use rust_apt::config::Config as AptConfig;
use serde::{Deserialize, Serialize};

use super::color::{setup_color, Color};
use super::{OptType, Paths, Switch};
use crate::config::color::{RatStyle, Style, Theme};
use crate::tui::progress::{NumSys, UnitStr};

#[derive(Serialize, Deserialize, Debug)]
/// Configuration struct
pub struct Config {
	#[serde(rename(deserialize = "Nala"), default)]
	map: HashMap<String, OptType>,

	#[serde(rename(deserialize = "Theme"), default)]
	pub(crate) theme: HashMap<Theme, Style>,

	// The following fields are not used with serde
	#[serde(skip)]
	pub apt: AptConfig,

	#[serde(skip)]
	/// The command that is being run
	pub command: String,
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

	pub fn rat_style<T: AsRef<Theme>>(&self, theme: T) -> RatStyle {
		self.theme
			.get(theme.as_ref())
			.unwrap_or(&Style::default())
			.to_rat()
	}

	pub fn rat_reset<T: AsRef<Theme>>(&self, theme: T) -> RatStyle {
		RatStyle::reset().set_style(self.rat_style(theme))
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
	pub fn load_args(&mut self, args: &ArgMatches) -> Result<()> {
		for alias in [
			("full-upgrade", "full"),
			("safe-upgrade", "safe"),
			("autopurge", "purge"),
			("purge", "purge"),
		] {
			if std::env::args().any(|arg| arg == alias.0) {
				self.map.insert(alias.1.to_string(), OptType::Bool(true));
			}
		}

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

		let switch = match self
			.map
			.get("color")
			.unwrap_or(&OptType::Switch(Switch::Auto))
		{
			OptType::Switch(switch) => *switch,
			_ => Switch::Auto,
		};

		setup_color(Color::new(switch, self.theme.clone()));

		if let Some(options) = self.get_vec("option") {
			for raw_opt in options {
				let Some((key, value)) = raw_opt.split_once("=") else {
					bail!("Option '{raw_opt}' is not supported");
				};
				self.apt.set(key, value);
			}
		}

		// If Debug is there we can print the whole thing.
		if self.debug() {
			dbg!(&self);
		}
		Ok(())
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

	pub fn get_mut_vec(&mut self, key: &str) -> Option<&mut Vec<String>> {
		if let OptType::VecString(vec) = self.map.get_mut(key)? {
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

	/// Retrieve the boolean value from the config
	/// additionally taking into account if `--no-option`
	/// has been passed on the cli to disable the feature.
	pub fn get_no_bool(&self, key: &str, default: bool) -> bool {
		let mut no_option = String::from("no_");
		no_option += key;
		if self.get_bool(&no_option, false) {
			return false;
		}
		self.get_bool(key, default)
	}

	/// Get the package names that were passed as arguments.
	pub fn pkg_names(&self) -> Result<Vec<String>> {
		let Some(pkg_names) = self.get_vec("pkg_names") else {
			bail!("You must specify a package");
		};

		let mut deduped = pkg_names.clone();
		deduped.dedup();
		deduped.sort();

		Ok(deduped)
	}

	pub fn arches(&self) -> Vec<String> {
		if self.get_bool("all_arches", false) {
			self.apt.get_architectures()
		} else {
			vec![self.apt.get_architectures().into_iter().next().unwrap()]
		}
	}

	/// Get the countries that were passed as arguments.
	pub fn countries(&self) -> Option<&Vec<String>> { self.get_vec("country") }

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

	pub fn allow_unauthenticated(&self) -> bool {
		self.get_bool("allow_unauthenticated", false)
			|| self.apt.bool("APT::Get::AllowUnauthenticated", false)
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
