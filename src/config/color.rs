use core::fmt;
use std::sync::{LazyLock, RwLock};

use crossterm::tty::IsTty;
use ratatui::style::{Color as RatColor, Modifier as RatMod, Style as RatStyle};
use ratatui::text::{Line, Span, Text};
use regex::Regex;
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use super::Switch;

static COLOR: LazyLock<RwLock<Color>> = LazyLock::new(|| RwLock::new(Color::default()));

pub fn setup_color(color: Color) {
	*COLOR.write().unwrap() = color;
}

/// Convenience function for non-macro callers/tests.
pub fn color_str<T: AsRef<Theme>, D: AsRef<str>>(theme: T, string: D) -> String {
	COLOR.read().unwrap().color(theme, string)
}

pub fn color_str_with_target<T: AsRef<Theme>, D: AsRef<str>>(
	theme: T,
	string: D,
	target: Target,
) -> String {
	COLOR
		.read()
		.unwrap()
		.color_with_target(theme, string, target)
}

#[macro_export]
macro_rules! color {
	($theme:expr, $string:expr) => {{
		$crate::config::color::color_str($theme, $string)
	}};
}

#[macro_export]
macro_rules! primary {
	($string:expr) => {{
		$crate::color!($crate::config::color::Theme::Primary, $string)
	}};
}

#[macro_export]
macro_rules! secondary {
	($string:expr) => {{
		$crate::color!($crate::config::color::Theme::Secondary, $string)
	}};
}

/// Hightlights the string according to configuration.
#[macro_export]
macro_rules! highlight {
	($string:expr) => {{
		$crate::color!($crate::config::color::Theme::Highlight, $string)
	}};
}

/// Color the version according to configuration.
#[macro_export]
macro_rules! ver {
	($string:expr) => {{
		let res = format!(
			"{}{}{}",
			$crate::highlight!("("),
			$crate::color!($crate::config::color::Theme::Secondary, $string),
			$crate::highlight!(")"),
		);
		res
	}};
}

pub use color;
pub use highlight;
pub use primary;
pub use secondary;
pub use ver;

#[derive(Clone, Copy, Serialize, Debug, PartialEq, Eq, Hash)]
pub enum ColorCode {
	Reset,
	Black,
	Red,
	Green,
	Yellow,
	Blue,
	Magenta,
	Cyan,
	Gray,
	DarkGray,
	LightRed,
	LightGreen,
	LightYellow,
	LightBlue,
	LightMagenta,
	LightCyan,
	White,
	Indexed(u8),
	Rgb(u8, u8, u8),
}

fn normalize_color_name(name: &str) -> String {
	name.chars()
		.filter(|c| !c.is_whitespace() && *c != '_' && *c != '-')
		.map(|c| c.to_ascii_lowercase())
		.collect()
}

fn parse_named_color(name: &str) -> Option<ColorCode> {
	let norm = normalize_color_name(name);
	Some(match norm.as_str() {
		"reset" => ColorCode::Reset,
		"black" => ColorCode::Black,
		"red" => ColorCode::Red,
		"green" => ColorCode::Green,
		"yellow" => ColorCode::Yellow,
		"blue" => ColorCode::Blue,
		"magenta" | "purple" => ColorCode::Magenta,
		"cyan" => ColorCode::Cyan,
		"gray" | "grey" => ColorCode::Gray,
		"darkgray" | "darkgrey" | "brightblack" => ColorCode::DarkGray,
		"lightred" | "brightred" => ColorCode::LightRed,
		"lightgreen" | "brightgreen" => ColorCode::LightGreen,
		"lightyellow" | "brightyellow" => ColorCode::LightYellow,
		"lightblue" | "brightblue" => ColorCode::LightBlue,
		"lightmagenta" | "brightmagenta" | "lightpurple" | "brightpurple" => {
			ColorCode::LightMagenta
		},
		"lightcyan" | "brightcyan" => ColorCode::LightCyan,
		"white" | "brightwhite" | "lightwhite" => ColorCode::White,
		"lightgray" | "lightgrey" => ColorCode::Gray,
		_ => return None,
	})
}

fn parse_hex_color(s: &str) -> Option<ColorCode> {
	let hex = s.strip_prefix('#')?;
	if hex.len() != 6 {
		return None;
	}

	let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
	let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
	let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
	Some(ColorCode::Rgb(r, g, b))
}

fn fg_bg_codes(color: ColorCode) -> Option<(&'static str, &'static str)> {
	Some(match color {
		ColorCode::Reset => ("0", "49"),
		ColorCode::Black => ("30", "40"),
		ColorCode::Red => ("31", "41"),
		ColorCode::Green => ("32", "42"),
		ColorCode::Yellow => ("33", "43"),
		ColorCode::Blue => ("34", "44"),
		ColorCode::Magenta => ("35", "45"),
		ColorCode::Cyan => ("36", "46"),
		ColorCode::Gray => ("37", "47"),
		ColorCode::DarkGray => ("90", "100"),
		ColorCode::LightRed => ("91", "101"),
		ColorCode::LightGreen => ("92", "102"),
		ColorCode::LightYellow => ("93", "103"),
		ColorCode::LightBlue => ("94", "104"),
		ColorCode::LightMagenta => ("95", "105"),
		ColorCode::LightCyan => ("96", "106"),
		ColorCode::White => ("97", "107"),
		ColorCode::Indexed(_) | ColorCode::Rgb(..) => return None,
	})
}

fn fg_ansi_code(color: ColorCode) -> String {
	match color {
		ColorCode::Indexed(i) => format!("38;5;{i}"),
		ColorCode::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
		_ => fg_bg_codes(color)
			.map(|(fg, _)| fg.to_string())
			.unwrap_or_else(|| "0".to_string()),
	}
}

fn bg_ansi_code(color: ColorCode) -> String {
	match color {
		ColorCode::Indexed(i) => format!("48;5;{i}"),
		ColorCode::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
		_ => fg_bg_codes(color)
			.map(|(_, bg)| bg.to_string())
			.unwrap_or_else(|| "49".to_string()),
	}
}

impl<'de> Deserialize<'de> for ColorCode {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct ColorCodeVisitor;

		impl<'de> Visitor<'de> for ColorCodeVisitor {
			type Value = ColorCode;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				f.write_str("a color name, 0-255 index, or #RRGGBB hex string")
			}

			fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				if value > u8::MAX as u64 {
					return Err(E::custom("color index must be between 0 and 255"));
				}
				Ok(ColorCode::Indexed(value as u8))
			}

			fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				if !(0..=u8::MAX as i64).contains(&value) {
					return Err(E::custom("color index must be between 0 and 255"));
				}
				Ok(ColorCode::Indexed(value as u8))
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				let value = value.trim();

				if let Some(hex) = parse_hex_color(value) {
					return Ok(hex);
				}

				if let Ok(index) = value.parse::<u8>() {
					return Ok(ColorCode::Indexed(index));
				}

				if let Some(color) = parse_named_color(value) {
					return Ok(color);
				}

				Err(E::custom(format!("unknown color '{value}'")))
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				self.visit_str(&value)
			}

			fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
			where
				M: MapAccess<'de>,
			{
				let mut parsed: Option<ColorCode> = None;
				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"Rgb" => {
							let vals: Vec<u8> = map.next_value()?;
							if vals.len() != 3 {
								return Err(de::Error::custom("Rgb expects three components"));
							}
							parsed = Some(ColorCode::Rgb(vals[0], vals[1], vals[2]));
						},
						"Indexed" => {
							let idx: u8 = map.next_value()?;
							parsed = Some(ColorCode::Indexed(idx));
						},
						other => {
							let _: de::IgnoredAny = map.next_value()?;
							return Err(de::Error::unknown_field(other, &["Rgb", "Indexed"]));
						},
					}
				}
				parsed.ok_or_else(|| de::Error::custom("expected Rgb or Indexed color"))
			}
		}

		deserializer.deserialize_any(ColorCodeVisitor)
	}
}

#[derive(Clone, Copy, Serialize, Debug, PartialEq, Eq)]
pub struct Modifiers(u16);

impl Modifiers {
	pub const BOLD: Self = Self(1 << 0);
	pub const CROSSED_OUT: Self = Self(1 << 8);
	pub const DIM: Self = Self(1 << 1);
	pub const HIDDEN: Self = Self(1 << 7);
	pub const ITALIC: Self = Self(1 << 2);
	pub const RAPID_BLINK: Self = Self(1 << 5);
	pub const REVERSED: Self = Self(1 << 6);
	pub const SLOW_BLINK: Self = Self(1 << 4);
	pub const UNDERLINED: Self = Self(1 << 3);

	pub const fn empty() -> Self {
		Self(0)
	}

	pub const fn bold() -> Self {
		Self::BOLD
	}

	pub fn contains(self, other: Self) -> bool {
		self.0 & other.0 == other.0
	}

	pub fn is_empty(self) -> bool {
		self.0 == 0
	}

	fn insert(&mut self, other: Self) {
		self.0 |= other.0
	}
}

fn bold() -> Modifiers {
	Modifiers::bold()
}

fn normalize_modifier(name: &str) -> String {
	name.chars()
		.filter(|c| !c.is_whitespace() && *c != '_' && *c != '-')
		.map(|c| c.to_ascii_uppercase())
		.collect()
}

fn parse_modifier_token(token: &str) -> Option<Modifiers> {
	let norm = normalize_modifier(token);
	Some(match norm.as_str() {
		"BOLD" => Modifiers::BOLD,
		"DIM" => Modifiers::DIM,
		"ITALIC" => Modifiers::ITALIC,
		"UNDERLINED" => Modifiers::UNDERLINED,
		"SLOWBLINK" => Modifiers::SLOW_BLINK,
		"RAPIDBLINK" => Modifiers::RAPID_BLINK,
		"REVERSED" => Modifiers::REVERSED,
		"HIDDEN" => Modifiers::HIDDEN,
		"CROSSEDOUT" | "STRIKETHROUGH" | "STRIKE" => Modifiers::CROSSED_OUT,
		_ => return None,
	})
}

fn parse_modifiers_from_str(s: &str) -> Option<Modifiers> {
	let mut mods = Modifiers::empty();
	let mut found = false;

	for token in s.split('|') {
		let token = token.trim();
		if token.is_empty() {
			continue;
		}
		let modifier = parse_modifier_token(token)?;
		mods.insert(modifier);
		found = true;
	}

	found.then_some(mods).or(Some(Modifiers::empty()))
}

impl<'de> Deserialize<'de> for Modifiers {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct ModVisitor;

		impl<'de> Visitor<'de> for ModVisitor {
			type Value = Modifiers;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				f.write_str("a modifier string like \"BOLD | ITALIC\" or an array")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				parse_modifiers_from_str(value)
					.ok_or_else(|| E::custom(format!("unknown modifier '{value}'")))
			}

			fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
			where
				E: de::Error,
			{
				self.visit_str(&value)
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: SeqAccess<'de>,
			{
				let mut mods = Modifiers::empty();

				while let Some(token) = seq.next_element::<String>()? {
					let modifier = parse_modifier_token(&token)
						.ok_or_else(|| de::Error::custom(format!("unknown modifier '{token}'")))?;
					mods.insert(modifier);
				}

				Ok(mods)
			}
		}

		deserializer.deserialize_any(ModVisitor)
	}
}

pub fn ansi_to_text(msg: &str) -> Text<'static> {
	let lines: Vec<Line> = msg
		.lines()
		.map(|msg| {
			let re = Regex::new(r"\x1b\[([0-9;]*)m").unwrap();
			let mut line = Line::default();
			let mut style = RatStyle::default();
			let mut pos = 0;

			for cap in re.captures_iter(msg) {
				let m = cap.get(0).unwrap();
				if m.start() > pos {
					line.push_span(Span::from(msg[pos..m.start()].to_string()).style(style));
				}
				style = ansi_to_style(style, cap.get(1).unwrap().as_str());
				pos = m.end();
			}

			if pos < msg.len() {
				line.push_span(Span::from(msg[pos..].to_string()).style(style));
			}

			line
		})
		.collect();
	Text::from(lines)
}

pub fn ansi_to_style(mut style: RatStyle, params: &str) -> RatStyle {
	let parts: Vec<&str> = params.split(';').collect();
	let mut i = 0;
	while i < parts.len() {
		match parts[i] {
			"" | "0" => style = RatStyle::default(),
			"1" => style = style.add_modifier(RatMod::BOLD),
			"2" => style = style.add_modifier(RatMod::DIM),
			"3" => style = style.add_modifier(RatMod::ITALIC),
			"4" => style = style.add_modifier(RatMod::UNDERLINED),
			"7" => style = style.add_modifier(RatMod::REVERSED),
			"9" => style = style.add_modifier(RatMod::CROSSED_OUT),
			"22" => style = style.remove_modifier(RatMod::BOLD | RatMod::DIM),
			"38" | "48" => {
				let set = |s: RatStyle, c: RatColor| {
					if parts[i] == "38" {
						s.fg(c)
					} else {
						s.bg(c)
					}
				};
				if i + 1 < parts.len() {
					match parts[i + 1] {
						"5" if i + 2 < parts.len() => {
							if let Ok(n) = parts[i + 2].parse() {
								style = set(style, RatColor::Indexed(n));
							}
							i += 2;
						},
						"2" if i + 4 < parts.len() => {
							let (r, g, b) = (
								parts[i + 2].parse().ok(),
								parts[i + 3].parse().ok(),
								parts[i + 4].parse().ok(),
							);
							if let (Some(r), Some(g), Some(b)) = (r, g, b) {
								style = set(style, RatColor::Rgb(r, g, b));
							}
							i += 4;
						},
						_ => {},
					}
				}
			},
			"39" => style = style.fg(RatColor::Reset),
			"49" => style = style.bg(RatColor::Reset),
			n if n.len() == 2 || n.len() == 3 => {
				if let Ok(n) = n.parse::<u8>() {
					match n {
						30..=37 => style = style.fg(RatColor::Indexed(n - 30)),
						40..=47 => style = style.bg(RatColor::Indexed(n - 40)),
						90..=97 => style = style.fg(RatColor::Indexed(n - 82)),
						100..=107 => style = style.bg(RatColor::Indexed(n - 92)),
						_ => {},
					}
				}
			},
			_ => {},
		}
		i += 1;
	}
	style
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Style {
	pub fg: ColorCode,
	pub bg: Option<ColorCode>,
	#[serde(default = "bold")]
	pub modifier: Modifiers,
}

impl Style {
	pub fn new(modifier: Modifiers, fg: ColorCode, bg: Option<ColorCode>) -> Self {
		Self { fg, bg, modifier }
	}

	pub fn default() -> Self {
		Self::no_bold(ColorCode::White)
	}

	pub fn bold(color: ColorCode) -> Self {
		Self::new(Modifiers::BOLD, color, None)
	}

	pub fn no_bold(color: ColorCode) -> Self {
		Self::new(Modifiers::empty(), color, None)
	}

	pub fn ansi_prefix(&self) -> String {
		let mut parts = vec![self.mod_string(), fg_ansi_code(self.fg)];
		if let Some(bg) = self.bg {
			parts.push(bg_ansi_code(bg));
		}
		format!("\x1b[{}m", parts.join(";"))
	}

	fn mod_string(&self) -> String {
		let mods = [
			(Modifiers::BOLD, "1"),
			(Modifiers::DIM, "2"),
			(Modifiers::ITALIC, "3"),
			(Modifiers::UNDERLINED, "4"),
			(Modifiers::SLOW_BLINK, "5"),
			(Modifiers::RAPID_BLINK, "6"),
			(Modifiers::REVERSED, "7"),
			(Modifiers::HIDDEN, "8"),
			(Modifiers::CROSSED_OUT, "9"),
		]
		.into_iter()
		.filter_map(|(m, a)| self.modifier.contains(m).then_some(a))
		.collect::<Vec<&str>>()
		.join(";");

		if mods.is_empty() {
			"0".to_string()
		} else {
			mods
		}
	}
}

impl Default for Style {
	fn default() -> Self {
		Self::no_bold(ColorCode::White)
	}
}

impl fmt::Display for Style {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.ansi_prefix())
	}
}

#[derive(Serialize, Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
pub enum Theme {
	Primary,
	Secondary,
	Highlight,
	Regular,
	ProgressFilled,
	ProgressUnfilled,
	Notice,
	Warning,
	Error,
}

impl AsRef<Theme> for Theme {
	fn as_ref(&self) -> &Theme {
		self
	}
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(default, rename_all = "PascalCase", deny_unknown_fields)]
pub struct ThemePalette {
	#[serde(default = "ThemePalette::default_primary")]
	pub primary: Style,
	#[serde(default = "ThemePalette::default_secondary")]
	pub secondary: Style,
	#[serde(default = "ThemePalette::default_highlight")]
	pub highlight: Style,
	#[serde(default = "ThemePalette::default_regular")]
	pub regular: Style,
	#[serde(default = "ThemePalette::default_progress_filled")]
	pub progress_filled: Style,
	#[serde(default = "ThemePalette::default_progress_unfilled")]
	pub progress_unfilled: Style,
	#[serde(default = "ThemePalette::default_notice")]
	pub notice: Style,
	#[serde(default = "ThemePalette::default_warning")]
	pub warning: Style,
	#[serde(default = "ThemePalette::default_error")]
	pub error: Style,
}

impl ThemePalette {
	fn default_primary() -> Style {
		Style::bold(ColorCode::LightGreen)
	}

	fn default_secondary() -> Style {
		Style::bold(ColorCode::LightBlue)
	}

	fn default_highlight() -> Style {
		Style::bold(ColorCode::White)
	}

	fn default_regular() -> Style {
		Style::no_bold(ColorCode::White)
	}

	fn default_progress_filled() -> Style {
		Style::bold(ColorCode::LightGreen)
	}

	fn default_progress_unfilled() -> Style {
		Style::bold(ColorCode::LightRed)
	}

	fn default_notice() -> Style {
		Style::bold(ColorCode::LightYellow)
	}

	fn default_warning() -> Style {
		Style::bold(ColorCode::LightYellow)
	}

	fn default_error() -> Style {
		Style::bold(ColorCode::LightRed)
	}

	pub fn style<T: AsRef<Theme>>(&self, theme: T) -> &Style {
		match theme.as_ref() {
			Theme::Primary => &self.primary,
			Theme::Secondary => &self.secondary,
			Theme::Highlight => &self.highlight,
			Theme::Regular => &self.regular,
			Theme::ProgressFilled => &self.progress_filled,
			Theme::ProgressUnfilled => &self.progress_unfilled,
			Theme::Notice => &self.notice,
			Theme::Warning => &self.warning,
			Theme::Error => &self.error,
		}
	}
}

impl Default for ThemePalette {
	fn default() -> Self {
		Self {
			primary: Self::default_primary(),
			secondary: Self::default_secondary(),
			highlight: Self::default_highlight(),
			regular: Self::default_regular(),
			progress_filled: Self::default_progress_filled(),
			progress_unfilled: Self::default_progress_unfilled(),
			notice: Self::default_notice(),
			warning: Self::default_warning(),
			error: Self::default_error(),
		}
	}
}

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(default)]
pub struct ColorConfig {
	pub mode: Switch,
	pub theme: ThemePalette,
}

impl ColorConfig {
	pub fn to_color(&self) -> Color {
		Color::new(self.mode, self.theme.clone())
	}

	pub fn with_mode(&self, mode: Switch) -> Color {
		Color::new(mode, self.theme.clone())
	}

	pub fn style<T: AsRef<Theme>>(&self, theme: T) -> &Style {
		self.theme.style(theme)
	}
}

pub struct Color {
	switch: Switch,
	theme: ThemePalette,
}

impl Color {
	pub fn new(switch: Switch, theme: ThemePalette) -> Color {
		Color { switch, theme }
	}

	pub fn theme(&self) -> &ThemePalette {
		&self.theme
	}

	pub fn can_color(&self, target: Target) -> bool {
		match self.switch {
			Switch::Always => true,
			Switch::Never => false,
			Switch::Auto => target.is_tty(),
		}
	}

	pub fn color<T: AsRef<Theme>, D: AsRef<str>>(&self, theme: T, string: D) -> String {
		self.color_with_target(theme, string, Target::Stdout)
	}

	pub fn color_with_target<T: AsRef<Theme>, D: AsRef<str>>(
		&self,
		theme: T,
		string: D,
		target: Target,
	) -> String {
		self.paint(self.theme.style(theme), string, target)
	}

	fn paint<D: AsRef<str>>(&self, style: &Style, string: D, target: Target) -> String {
		let string = string.as_ref();
		if self.can_color(target) {
			return format!("{style}{string}\x1b[0m");
		}
		string.to_string()
	}
}

impl Default for Color {
	fn default() -> Self {
		Self::new(Switch::Auto, ThemePalette::default())
	}
}

#[derive(Clone, Copy)]
pub enum Target {
	Stdout,
	Stderr,
}

impl Target {
	fn is_tty(self) -> bool {
		match self {
			Self::Stdout => std::io::stdout().is_tty(),
			Self::Stderr => std::io::stderr().is_tty(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn color_resets_when_enabled() {
		let color = Color::new(Switch::Always, ThemePalette::default());
		let out = color.color_with_target(Theme::Primary, "hi", Target::Stdout);
		assert!(out.starts_with("\u{1b}["));
		assert!(out.ends_with("\u{1b}[0m"));
		assert!(out.contains("hi"));
	}

	#[test]
	fn color_is_passthrough_when_disabled() {
		let color = Color::new(Switch::Never, ThemePalette::default());
		let out = color.color_with_target(Theme::Primary, "hi", Target::Stdout);
		assert_eq!(out, "hi");
	}

	#[test]
	fn parse_named_and_hex_colors() {
		let bright: ColorCode = serde_json::from_str("\"BrightGreen\"").unwrap();
		let hex: ColorCode = serde_json::from_str("\"#002B36\"").unwrap();
		let indexed: ColorCode = serde_json::from_str("201").unwrap();

		assert_eq!(bright, ColorCode::LightGreen);
		assert_eq!(hex, ColorCode::Rgb(0x00, 0x2B, 0x36));
		assert_eq!(indexed, ColorCode::Indexed(201));
	}

	#[test]
	fn parse_modifiers_from_string_and_array() {
		let pipe: Modifiers = serde_json::from_str("\"BOLD | ITALIC\"").unwrap();
		let array: Modifiers = serde_json::from_str("[\"UNDERLINED\", \"BOLD\"]").unwrap();

		assert!(pipe.contains(Modifiers::BOLD));
		assert!(pipe.contains(Modifiers::ITALIC));
		assert!(array.contains(Modifiers::UNDERLINED));
		assert!(array.contains(Modifiers::BOLD));
	}

	#[test]
	fn style_prefix_includes_background_codes() {
		let style = Style::new(
			Modifiers::BOLD,
			ColorCode::Indexed(201),
			Some(ColorCode::Indexed(125)),
		);

		let prefix = style.ansi_prefix();
		assert!(prefix.starts_with("\u{1b}["));
		assert!(prefix.contains("38;5;201"));
		assert!(prefix.contains("48;5;125"));
	}
}
