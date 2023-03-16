use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use clap::{ArgMatches, ValueSource};
use serde::Deserialize;

use crate::colors::{Color, ColorType, Style, Theme, COLOR_MAP};
use crate::util::dprint;

#[derive(Deserialize, Debug)]
/// Configuration struct
pub struct Config {
	#[serde(rename(deserialize = "Nala"))]
	nala_map: HashMap<String, bool>,
	#[serde(rename(deserialize = "Theme"))]
	color_data: HashMap<String, ThemeType>,

	// The following fields are not used with serde
	#[serde(skip)]
	pub color: Color,

	#[serde(skip)]
	pkg_names: Option<Vec<String>>,
}

impl Default for Config {
	/// The default configuration for Nala.
	fn default() -> Config {
		Config {
			nala_map: HashMap::new(),
			color_data: HashMap::new(),
			color: Color::default(),
			pkg_names: None,
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
	pub fn load_args(&mut self, args: &ArgMatches) {
		let bool_opts = [
			"debug",
			"verbose",
			"description",
			"summary",
			"all-versions",
			"installed",
			"nala-installed",
			"upgradable",
			"virtual",
			"names",
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
			if self.nala_map.get(opt).is_none() {
				// set it to false
				self.set_bool(opt, false)
			}
		}

		// TODO: I bet this breaks on commands without pkgnames.
		// See the first condition in this loop
		if let Some(pkg_names) = args.get_many::<String>("pkg-names") {
			let pkgs: Vec<String> = pkg_names.cloned().collect();
			self.pkg_names = if pkgs.is_empty() { None } else { Some(pkgs) };

			dprint!(self, "Package Names = {:?}", self.pkg_names);
		}

		// If Debug is there we can print the whole thing.
		if self.debug() {
			let map_string = format!("Config Map = {:#?}", self.nala_map);
			for line in map_string.lines() {
				eprintln!("DEBUG: {line}")
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

	/// Get the package names that were passed as arguments
	pub fn pkg_names(&self) -> Option<&Vec<String>> { self.pkg_names.as_ref() }

	/// Return true if debug is enabled
	pub fn debug(&self) -> bool { self.get_bool("debug", false) }

	fn update_color(&mut self) -> Result<()> {
		let default_map = &*COLOR_MAP;
		// Key will be the name of the format, example: "error"
		for key in default_map.keys() {
			// If the key is not in the defaults, ignore it
			let theme = match self.color_data.get(*key) {
				Some(theme) => theme,
				// We probably should set a default here from the default map?
				None => continue,
			};

			self.color.color_map.insert(
				*key,
				Theme::new(
					match &theme.style {
						SerdeStyle::Text(string) => Style::from_str(string)?,
						SerdeStyle::Integer(int) => Style::from_u8(int)?,
						SerdeStyle::Array(vector) => Style::from_array(vector)?,
					},
					match &theme.color {
						SerdeColor::Text(string) => ColorType::from_str(string)?,
						SerdeColor::Integer(int) => ColorType::from_u8(int),
						SerdeColor::Array(array) => ColorType::from_array(array),
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
