use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use anyhow::{anyhow, Context, Result};
use lazy_static::lazy_static;
use toml::Value;

use crate::config::Config;
use crate::util::dprint;

static RESET: &str = "\x1b[0m";

lazy_static! {
	static ref COLOR_MAP: HashMap<&'static str, u8> = HashMap::from([
		("black", 0),
		("red", 1),
		("green", 2),
		("yellow", 3),
		("blue", 4),
		("magenta", 5),
		("cyan", 6),
		("white", 7),
		("bright_black", 8),
		("bright_red", 9),
		("bright_green", 10),
		("bright_yellow", 11),
		("bright_blue", 12),
		("bright_magenta", 13),
		("bright_cyan", 14),
		("bright_white", 15),
		("warning", 11),
		("error", 9),
		("package", 10),
		("version", 12),
	]);
}

#[repr(u8)]
#[derive(Clone, Debug)]
/// Ansi Color Styles
pub enum Style {
	/// Text is Normal
	Normal,
	/// Text is Bold
	Bold,
	/// Text is Faint
	Faint,
	/// Text is Italic
	Italic,
	/// Underlines the text
	Underline,
	/// Text will slowly blink
	SlowBlink,
	/// Rapidly blinks the text
	RapidBlink,
	/// Inverts the Foreground and Background Colors
	InvertColors,
	/// Not widely supported
	Hide,
	/// Strike through the text
	StrikeThrough,
	/// Multiple Styles as a String
	Multiple(String),
}

impl Style {
	/// Return the Enum as a str ansi code
	///
	/// If called on `Style::Multiple` this will panic
	pub fn as_str(&self) -> &'static str {
		match self {
			Style::Normal => "0",
			Style::Bold => "1",
			Style::Faint => "2",
			Style::Italic => "3",
			Style::Underline => "4",
			Style::SlowBlink => "5",
			Style::RapidBlink => "6",
			Style::InvertColors => "7",
			Style::Hide => "8",
			Style::StrikeThrough => "9",
			_ => panic!("as_str is not supported for the multiple variant"),
		}
	}

	/// Load a Style from an int such as `1` for Bold
	pub fn from_i64(value: &i64) -> Result<Style> {
		match value {
			0 => Ok(Style::Normal),
			1 => Ok(Style::Bold),
			2 => Ok(Style::Faint),
			3 => Ok(Style::Italic),
			4 => Ok(Style::Underline),
			5 => Ok(Style::SlowBlink),
			6 => Ok(Style::RapidBlink),
			7 => Ok(Style::InvertColors),
			8 => Ok(Style::Hide),
			9 => Ok(Style::StrikeThrough),
			_ => Err(anyhow!("Value '{value}' is not a valid Int Style")),
		}
	}

	/// Load a Style from a str such as `"underline"`
	pub fn from_str(value: &str) -> Result<Style> {
		match value {
			"default" => Ok(Style::Normal),
			"bold" => Ok(Style::Bold),
			"faint" => Ok(Style::Faint),
			"italic" => Ok(Style::Italic),
			"underline" => Ok(Style::Underline),
			"slow_blink" => Ok(Style::SlowBlink),
			"rapid_blink" => Ok(Style::RapidBlink),
			"invert_colors" => Ok(Style::InvertColors),
			"hide" => Ok(Style::Hide),
			"strike_through" => Ok(Style::StrikeThrough),
			_ => Err(anyhow!("Value '{value}' is not a valid String Style")),
		}
	}

	/// Load a style from a toml array
	///
	/// Return `Style::Multiple(String)`
	pub fn from_toml_array(vector: &Vec<Value>) -> Result<Style> {
		let last = vector.len() - 1;
		let mut string = String::new();
		for (i, value) in vector.iter().enumerate() {
			let style = match value {
				Value::Integer(int) => Style::from_i64(int)?,
				Value::String(string) => Style::from_str(string)?,
				_ => return Err(anyhow!("RGB Value should be 'int' not '{value:?}'")),
			};

			string += style.as_str();
			if i != last {
				string += ";"
			}
		}
		Ok(Style::Multiple(string))
	}
}

impl fmt::Display for Style {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			// Multiple cannot use as_str. It will panic
			Style::Multiple(string) => write!(f, "{string}")?,
			_ => write!(f, "{}", self.as_str())?,
		}
		Ok(())
	}
}

#[derive(Debug)]
pub enum ColorType {
	Standard(u8),
	Rgb(String),
}

impl ColorType {
	pub fn from_i64(value: &i64) -> Result<ColorType> {
		let num = u8::try_from(*value).with_context(|| "Int must be in range 0...255")?;
		Ok(ColorType::Standard(num))
	}

	pub fn from_str(value: &str) -> Result<ColorType> {
		match value {
			"black" => Ok(ColorType::Standard(0)),
			"red" => Ok(ColorType::Standard(1)),
			"green" => Ok(ColorType::Standard(2)),
			"yellow" => Ok(ColorType::Standard(3)),
			"blue" => Ok(ColorType::Standard(4)),
			"magenta" => Ok(ColorType::Standard(5)),
			"cyan" => Ok(ColorType::Standard(6)),
			"white" => Ok(ColorType::Standard(7)),
			"bright_black" => Ok(ColorType::Standard(8)),
			"bright_red" => Ok(ColorType::Standard(9)),
			"bright_green" => Ok(ColorType::Standard(10)),
			"bright_yellow" => Ok(ColorType::Standard(11)),
			"bright_blue" => Ok(ColorType::Standard(12)),
			"bright_magenta" => Ok(ColorType::Standard(13)),
			"bright_cyan" => Ok(ColorType::Standard(14)),
			"bright_white" => Ok(ColorType::Standard(15)),
			_ => Err(anyhow!("Value '{value}' is not a valid Color")),
		}
	}

	pub fn from_toml_array(vector: &Vec<Value>) -> Result<ColorType> {
		let total = vector.len();
		if total != 3 {
			return Err(anyhow!("RGB Value should contain 3 integers not {total}"));
		}

		let mut string = String::new();
		for (num, value) in vector.iter().enumerate() {
			match value {
				Value::Integer(int) => {
					let rgb = u8::try_from(*int).with_context(|| "Int must be in range 0...255")?;
					string += &rgb.to_string();
					if num != 2 {
						string += ";";
					}
				},
				_ => {
					return Err(anyhow!("RGB Value should be 'int' not '{value:?}'"));
				},
			}
		}
		Ok(ColorType::Rgb(string))
	}
}

impl fmt::Display for ColorType {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			ColorType::Standard(int) => write!(f, "{int}"),
			ColorType::Rgb(string) => write!(f, "{string}"),
		}
	}
}

#[derive(Debug)]
pub struct Theme {
	#[allow(dead_code)]
	/// Name is used for debug purposes
	name: &'static str,
	style: Style,
	color: ColorType,
}

impl Theme {
	pub fn new(name: &'static str, style: Style, color: ColorType) -> Self {
		Self { name, style, color }
	}

	pub fn default(name: &'static str, color: ColorType) -> Self {
		Self {
			name,
			style: Style::Bold,
			color,
		}
	}
}

/// Color text based on Style and ColorCodes
pub struct Color {
	can_color: bool,
	color_map: HashMap<&'static str, Theme>,
}

impl Color {
	pub fn default() -> Color {
		let mut color_map: HashMap<&'static str, Theme> = HashMap::new();
		for (color, code) in &*COLOR_MAP {
			color_map.insert(color, Theme::default(color, ColorType::Standard(*code)));
		}
		Color {
			can_color: true,
			color_map,
		}
	}

	pub fn update_from_config(&mut self, config: &Config) -> Result<()> {
		let config_map = match config.color_map.as_ref() {
			Ok(map) => map,
			Err(err) => {
				self.warn(&format!("{err:?}"));
				return Ok(());
			},
		};

		// Iterate the key, value of the [Theme] Configuration
		for (string, value) in config_map.iter() {
			let conf_key = string.as_str();

			// If the key is not in the defaults, ignore it
			if !self.color_map.contains_key(conf_key) {
				continue;
			}

			match value {
				// Each Theme should be a table
				Value::Table(table) => {
					dprint!(config, "Loading '{conf_key}' from config");
					let (key, _unused) = self.color_map.get_key_value(conf_key).unwrap();
					let theme = Theme::new(
						key,
						config
							.get_style(table, "style")
							.with_context(|| format!("Invalid value for '{key}.style'"))?,
						config
							.get_color(table, "color", *COLOR_MAP.get(key).unwrap())
							.with_context(|| format!("Invalid value for '{key}.color'"))?,
					);
					dprint!(config, "Loaded {theme:?}");
					self.color_map.insert(key, theme);
				},
				_ => {
					return Err(anyhow!(
						"Invalid value for '{conf_key}': Unsupported Type '{}'",
						value.type_str()
					))
				},
			}
		}
		Ok(())
	}

	/// Color a string based on a Theme
	pub fn color<'a>(&self, theme: &Theme, string: &'a str) -> Cow<'a, str> {
		if self.can_color {
			let style = &theme.style;
			match &theme.color {
				ColorType::Standard(color) => {
					return Cow::Owned(format!("\x1b[{style};38;5;{color}m{string}{RESET}"));
				},
				ColorType::Rgb(color) => {
					return Cow::Owned(format!("\x1b[{style};38;2;{color}m{string}{RESET}"));
				},
			}
		}
		Cow::Borrowed(string)
	}

	pub fn style<'a>(&self, style: Style, string: &'a str) -> Cow<'a, str> {
		if self.can_color {
			return Cow::Owned(format!("\x1b[{style}m{string}{RESET}"));
		}
		Cow::Borrowed(string)
	}

	// /// Color the text red with configured settings
	// pub fn red<'a>(&self, string: &'a str) -> Cow<'a, str> {
	// 	self.color(self.color_map.get("bright_red").unwrap(), string)
	// }

	/// Color the text red with configured settings
	pub fn yellow<'a>(&self, string: &'a str) -> Cow<'a, str> {
		self.color(self.color_map.get("bright_yellow").unwrap(), string)
	}

	// pub fn blue<'a>(&self, string: &'a str) -> Cow<'a, str> {
	// 	self.color(self.color_map.get("bright_blue").unwrap(), string)
	// }

	/// Styles the text in bold only
	pub fn bold<'a>(&self, string: &'a str) -> Cow<'a, str> { self.style(Style::Bold, string) }

	/// Color the package name according to configuration
	pub fn package<'a>(&self, string: &'a str) -> Cow<'a, str> {
		self.color(self.color_map.get("package").unwrap(), string)
	}

	/// Color the version according to configuration
	pub fn version<'a>(&self, string: &'a str) -> Cow<'a, str> {
		let open = self.bold("(");
		let close = self.bold(")");
		let version = self.color(self.color_map.get("version").unwrap(), string);
		Cow::Owned(format!("{open}{version}{close}"))
	}

	/// Print a warning to stderr
	pub fn warn(&self, string: &str) {
		eprintln!(
			"{} {string}",
			self.color(self.color_map.get("warning").unwrap(), "Warning:",)
		);
	}

	/// Print an error to stderr
	pub fn error(&self, string: &str) {
		eprintln!(
			"{} {string}",
			self.color(self.color_map.get("error").unwrap(), "Error:",)
		);
	}
}
