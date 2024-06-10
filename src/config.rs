use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use rust_apt::config::Config as AptConfig;
use serde::Deserialize;

use crate::cli::Commands;
use crate::colors::{Color, ColorType, Style, Theme, COLOR_MAP};
use crate::util::dprint;

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
}

impl Paths {
	pub fn path(&self) -> &'static str {
		match self {
			Paths::Archive => "Dir::Cache::Archives",
			Paths::Lists => "Dir::State::Lists",
			Paths::SourceList => "Dir::Etc::sourcelist",
			Paths::SourceParts => "Dir::Etc::sourceparts",
			Paths::NalaSources => "/etc/apt/sources.list.d/nala.sources",
		}
	}

	pub fn default_path(&self) -> &'static str {
		match self {
			Paths::Archive => "/var/cache/apt/archives/",
			Paths::Lists => "/var/lib/apt/lists/",
			Paths::SourceList => "/etc/apt/sources.list",
			Paths::SourceParts => "/etc/apt/sources.list.d/",
			Paths::NalaSources => "/etc/apt/sources.list.d/nala.sources",
		}
	}
}

#[derive(Deserialize, Debug)]
/// Configuration struct
pub struct Config {
	#[serde(rename(deserialize = "Nala"), default)]
	nala_map: HashMap<String, bool>,
	#[serde(rename(deserialize = "Theme"), default)]
	color_data: HashMap<String, ThemeType>,

	#[serde(skip)]
	pub string_map: HashMap<String, String>,

	// The following fields are not used with serde
	#[serde(skip)]
	pub color: Color,

	#[serde(skip)]
	pkg_names: Option<Vec<String>>,

	#[serde(skip)]
	countries: Option<Vec<String>>,

	#[serde(skip)]
	pub apt: AptConfig,

	#[serde(skip)]
	pub auto: Option<u8>,

	#[serde(skip)]
	/// The command the is being run
	pub command: String,
}

impl Default for Config {
	/// The default configuration for Nala.
	fn default() -> Config {
		Config {
			nala_map: HashMap::new(),
			string_map: HashMap::new(),
			color_data: HashMap::new(),
			color: Color::default(),
			pkg_names: None,
			countries: None,
			auto: None,
			apt: AptConfig::new(),
			command: "Command Not Given Yet".to_string(),
		}
	}
}

impl Config {
	pub fn new(conf_file: &Path) -> Result<Config> {
		// Try to read the entire config file and map it.
		// Return an empty config and print a warning on failure.

		// Eventually this needs to include preinstall and postinstall sections.
		let mut map = Self::read_config(conf_file)?;
		map.update_color()?;
		Ok(map)
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
	pub fn load_args(&mut self, args: &ArgMatches, commands: Option<Commands>) {
		if let Some(Commands::Fetch(opts)) = commands {
			self.auto = opts.auto;
		};

		let bool_opts = [
			"debug",
			"verbose",
			"description",
			"summary",
			"all_versions",
			"installed",
			"nala_installed",
			"upgradable",
			"virtual",
			"names",
			"lists",
			"fetch",
			// Fetch Options
			"non_free",
			"https_only",
			"sources",
		];

		for opt in bool_opts {
			// Clap seems to work differently in a release build
			// For a debug build we need to check for an error
			if args.try_get_one::<bool>(opt).is_err() {
				self.set_bool(opt, false);
				continue;
			}

			// If the flag exists
			if let Some(value) = args.get_one::<bool>(opt) {
				// And the flag was passed from the command line
				if let Some(ValueSource::CommandLine) = args.value_source(opt) {
					// Set the config
					self.set_bool(opt, *value);
					continue;
				}
			}

			// If the flag doesn't exist, wasn't passed by the user,
			// and isn't present in the config
			if !self.nala_map.contains_key(opt) {
				// set it to false
				self.set_bool(opt, false);
			}
		}

		if let Ok(Some(pkg_names)) = args.try_get_many::<String>("pkg_names") {
			let pkgs: Vec<String> = pkg_names.cloned().collect();
			self.pkg_names = if pkgs.is_empty() { None } else { Some(pkgs) };

			dprint!(self, "Package Names = {:?}", self.pkg_names);
		}

		// TODO: It may be time to make these like bool opts above.
		if let Ok(Some(countries)) = args.try_get_many::<String>("country") {
			let country_vec: Vec<String> = countries.cloned().collect();
			self.countries = if country_vec.is_empty() { None } else { Some(country_vec) };

			dprint!(self, "Country = {:?}", self.countries);
		}

		let option_strings = ["debian", "ubuntu", "devuan"];

		for opt in option_strings {
			if let Ok(Some(value)) = args.try_get_one::<String>(opt) {
				self.string_map.insert(opt.to_string(), value.to_string());
			}
		}

		// If Debug is there we can print the whole thing.
		if self.debug() {
			let map_string = format!("Config Map = {:#?}", self.nala_map);
			for line in map_string.lines() {
				eprintln!("DEBUG: {line}");
			}
		}
	}

	/// Get a bool from the configuration by &str
	pub fn get_bool(&self, key: &str, default: bool) -> bool {
		match self.nala_map.get(key) {
			Some(value) => *value,
			_ => default,
		}
	}

	/// Set a bool in the configuration
	pub fn set_bool(&mut self, key: &str, value: bool) {
		self.nala_map.insert(key.to_string(), value);
	}

	/// Get a file from the configuration based on the Path enum.
	pub fn get_file(&self, file: &Paths) -> String {
		match file {
			// For now NalaSources is hard coded.
			Paths::NalaSources => file.path().to_string(),
			_ => self.apt.file(file.path(), file.default_path()),
		}
	}

	/// Get a path from the configuration based on the Path enum.
	pub fn get_path(&self, dir: &Paths) -> String {
		match dir {
			// For now NalaSources is hard coded.
			Paths::NalaSources => dir.path().to_string(),
			// Everything else should be an Apt Path
			_ => self.apt.dir(dir.path(), dir.default_path()),
		}
	}

	/// Get the package names that were passed as arguments
	pub fn pkg_names(&self) -> Option<&Vec<String>> { self.pkg_names.as_ref() }

	/// Get the countries that were passed as arguments
	pub fn countries(&self) -> Option<&Vec<String>> { self.countries.as_ref() }

	/// Return true if debug is enabled
	pub fn debug(&self) -> bool { self.get_bool("debug", false) }

	fn update_color(&mut self) -> Result<()> {
		let default_map = &*COLOR_MAP;
		// Key will be the name of the format, example: "error"
		for key in default_map.keys() {
			// If the key is not in the defaults, ignore it
			let Some(theme) = self.color_data.get(*key) else {
				continue;
			};

			self.color.color_map.insert(
				*key,
				Theme::new(
					match &theme.style {
						SerdeStyle::Text(string) => Style::from_str(string)?,
						SerdeStyle::Integer(int) => Style::from_u8(*int)?,
						SerdeStyle::Array(vector) => Style::from_array(vector)?,
					},
					match &theme.color {
						SerdeColor::Text(string) => ColorType::from_str(string)?,
						SerdeColor::Integer(int) => ColorType::from_u8(*int),
						SerdeColor::Array(array) => ColorType::from_array(*array),
					},
				),
			);
		}
		Ok(())
	}
}

// Transitional structs to go from Serde parse into the Color Struct
// It may be worth considering a way to marry the two.
// ATM they are separated due to additional type checking that seems complicated
// with serde

#[derive(Deserialize, Debug)]
struct ThemeType {
	style: SerdeStyle,
	color: SerdeColor,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum SerdeStyle {
	Text(String),
	Integer(u8),
	Array(Vec<String>),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum SerdeColor {
	Text(String),
	Integer(u8),
	Array([u8; 3]),
}
