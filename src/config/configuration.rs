use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use rust_apt::config::Config as AptConfig;

use super::color::Color;
use super::logger::{Logger, Writer};
use super::{Opt, Paths};
use crate::tui::progress::{NumSys, UnitStr};

macro_rules! take_arg {
	($map:expr, $args:expr, $key:ident -> $ty:ty) => {
		if let Ok(Some(value)) = $args.try_get_one::<$ty>(&$key) {
			$map.insert($key, value.into());
			continue;
		}
	};
}

macro_rules! vec_arg {
	($map:expr, $args:expr, $key:ident -> $ty:ty) => {
		if let Ok(Some(value)) = $args.try_get_occurrences::<$ty>(&$key) {
			$map.insert($key, Opt::from_iter(value.flatten().cloned()));
			continue;
		}
	};
}

#[derive(Debug)]
/// Configuration struct
pub struct Config {
	pub color: Color,

	map: HashMap<String, Opt>,

	pub apt: AptConfig,

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
	fn default() -> Config { Config::new(Color::default(), HashMap::new()) }
}

impl Config {
	fn new(color: Color, map: HashMap<String, Opt>) -> Config {
		Config {
			color,
			map,
			apt: AptConfig::new(),
			command: "Command Not Given Yet".to_string(),
		}
	}

	/// Read and Return the entire toml configuration file
	pub fn read_config(conf_file: &Path) -> Config {
		let color = Color::from_config();
		let _ = Logger::new(color, Writer::default()).init();

		let Ok(file) = fs::read_to_string(conf_file) else {
			return Config::default();
		};

		Config::new(
			Color::from_config(),
			toml::from_str(&file).unwrap_or_default(),
		)
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
				self.map.insert(alias.1.to_string(), Opt::Bool(true));
			}
		}

		for id in args.ids() {
			let key = id.as_str().to_string();
			// Don't do anything if the option wasn't specifically passed
			if Some(ValueSource::CommandLine) != args.value_source(&key) {
				continue;
			}

			take_arg!(self.map, args, key -> bool);
			vec_arg!(self.map, args, key -> String);
			take_arg!(self.map, args, key -> u8);
			take_arg!(self.map, args, key -> u64);
		}

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
			crate::debug!("{self:?}");
		}
		Ok(())
	}

	/// Get a bool from the configuration.
	pub fn get_bool(&self, key: &str, default: bool) -> bool {
		if let Some(Opt::Bool(bool)) = self.map.get(key) {
			return *bool;
		}
		default
	}

	/// Set a bool in the configuration.
	pub fn set_bool(&mut self, key: &str, value: bool) {
		self.map.insert(key.to_string(), Opt::Bool(value));
	}

	/// Get a single str from the configuration.
	pub fn get_str(&self, key: &str) -> Option<&str> {
		if let Opt::VecString(vec) = self.map.get(key)? {
			return vec.first().map(|x| x.as_str());
		}

		if let Opt::String(str) = self.map.get(key)? {
			return Some(str);
		}
		None
	}

	/// Get a Vec of Strings from the configuration.
	pub fn get_vec(&self, key: &str) -> Option<&Vec<String>> {
		if let Opt::VecString(vec) = self.map.get(key)? {
			return Some(vec);
		}
		None
	}

	pub fn get_mut_vec(&mut self, key: &str) -> Option<&mut Vec<String>> {
		if let Opt::VecString(vec) = self.map.get_mut(key)? {
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
		if let Opt::Int(value) = self.map.get("auto")? {
			return Some(*value);
		}
		None
	}

	pub fn unit_str(&self, unit: u64) -> String {
		if let Some(Opt::UnitStr(value)) = self.map.get("UnitStr") {
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

// fn from_user(opt: Option<ValueSource>) -> bool {
// 	// Don't do anything if the option wasn't specifically passed
// 	if Some(ValueSource::CommandLine) == opt {
// 		return true;
// 	}
// 	false
// }

#[cfg(test)]
mod test {
	use std::collections::HashMap;

	use crate::config::Theme;
	use crate::tui::progress::{NumSys, UnitStr};

	#[test]
	fn serialize_config() {
		let mut config = HashMap::new();
		config.insert(
			"unit_str".to_string(),
			super::Opt::UnitStr(UnitStr::new(0, NumSys::Binary)),
		);

		let toml = toml::to_string_pretty(&config).unwrap();
		println!("{toml}")
	}

	#[test]
	fn serialize_theme() {
		let theme = Theme::style_map();
		let toml = toml::to_string_pretty(&theme).unwrap();

		println!("{toml}")
	}
}
