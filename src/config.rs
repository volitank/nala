use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use clap::{ArgMatches, ValueSource};
use toml::map::Map;
use toml::Value;

use crate::colors::{Color, ColorType, Style};
use crate::util::dprint;

#[derive(Debug)]
/// Configuration struct
pub struct Config {
	nala_map: Map<String, Value>,
	pkg_names: Option<Vec<String>>,
	pub color_map: Result<Map<String, Value>>,
}

impl Default for Config {
	/// The default configuration for Nala.
	fn default() -> Config {
		Config {
			nala_map: Map::new(),
			pkg_names: None,
			color_map: Err(anyhow!("Default is assumed")),
		}
	}
}

impl Config {
	pub fn error(err: anyhow::Error) -> Self {
		Config {
			nala_map: Map::new(),
			pkg_names: None,
			color_map: Err(err),
		}
	}

	pub fn new(color: &Color, conf_file: &Path) -> Config {
		// Try to read the entire config file and map it.
		// Return an empty config and print a warning on failure.
		let config = match Self::read_config(conf_file) {
			Ok(map) => map,
			Err(err) => {
				color.warn(&format!("{err:?}"));
				return Config::error(err);
			},
		};

		// Reads the [Nala] section of the config
		let nala_map = Self::read_section(&config, conf_file, "Nala").unwrap_or_else(|err| {
			color.warn(&format!("{err:?}"));
			Map::new()
		});

		// Result is unused.
		// It will be handled when the Color struct is loaded from the config.
		let color_map = Self::read_section(&config, conf_file, "Theme");

		// Eventually this needs to include preinstall and postinstall sections.
		Config {
			nala_map,
			pkg_names: None,
			color_map,
		}
	}

	/// Read and Return the entire toml configuration file
	fn read_config(conf_file: &Path) -> Result<Map<String, Value>> {
		let conf = fs::read_to_string(conf_file)
			.with_context(|| format!("Failed to read {}, using defaults", conf_file.display()))?
			// Parse the Toml string
			.parse::<Value>()
			.with_context(|| format!("Failed to parse {}, using defaults", conf_file.display()))?
			// Serialize Toml into a Mapping
			.try_into::<Map<String, Value>>()
			.with_context(|| {
				format!(
					"Unable to serialize {}, using defaults",
					conf_file.display()
				)
			})?;
		Ok(conf)
	}

	/// Read and Return a specific section of the configuration file
	fn read_section(
		config_map: &Map<String, Value>,
		conf_file: &Path,
		section: &str,
	) -> Result<Map<String, Value>> {
		let section_map = config_map
			.get(section)
			.with_context(|| {
				format!(
					"Section '[{section}]' was not found in {}, using defaults",
					conf_file.display()
				)
			})?
			// Clone and Serialize the Nala section into its own Map
			.clone()
			.try_into::<Map<String, Value>>()
			.with_context(|| {
				format!(
					"Unable to map '[{section}]' from {}, using defaults",
					conf_file.display()
				)
			})?;
		Ok(section_map)
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

			// If the flag doesn't exist, wasn't passed by the user, and isn't present in
			// the config
			if self.nala_map.get(opt).is_none() {
				// set it to false
				self.set_bool(opt, false)
			}
		}

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
			Some(Value::Boolean(ret)) => *ret,
			_ => default,
		}
	}

	/// Set a bool in the configuration
	pub fn set_bool(&mut self, key: &str, value: bool) {
		self.nala_map.insert(key.to_string(), Value::Boolean(value));
	}

	/// Get the package names that were passed as arguments
	pub fn pkg_names(&self) -> Option<&Vec<String>> { self.pkg_names.as_ref() }

	/// Return true if debug is enabled
	pub fn debug(&self) -> bool { self.get_bool("debug", false) }

	/// Get the color information from the configuration
	pub fn get_color(
		&self,
		theme_map: &Map<String, Value>,
		key: &str,
		default: u8,
	) -> Result<ColorType> {
		if let Some(value) = theme_map.get(key) {
			let color = match value {
				Value::Integer(int) => ColorType::from_i64(int)?,
				Value::String(string) => ColorType::from_str(string)?,
				Value::Array(array) => ColorType::from_toml_array(array)?,
				_ => {
					return Err(anyhow!("Unsupported Type '{}'", value.type_str()));
				},
			};

			dprint!(self, "Loading '{key}' from config {value:?} as {color:?}");
			return Ok(color);
		}
		// Return the default
		let color = ColorType::Standard(default);
		dprint!(self, "Key: '{key}' not found, using default '{color:?}'");
		Ok(color)
	}

	/// Get the style information from the configuration
	pub fn get_style(&self, theme_map: &Map<String, Value>, key: &str) -> Result<Style> {
		if let Some(value) = theme_map.get(key) {
			let style = match value {
				Value::Integer(int) => Style::from_i64(int)?,
				Value::String(string) => Style::from_str(string)?,
				Value::Array(array) => Style::from_toml_array(array)?,
				_ => {
					return Err(anyhow!("Unsupported Type '{}'", value.type_str()));
				},
			};

			dprint!(self, "Loading '{key}' from config {value:?} as {style:?}");
			return Ok(style);
		}
		// Return the default
		let style = Style::Bold;
		dprint!(self, "Key: '{key}' not found, using default '{style:?}'");
		Ok(style)
	}
}
