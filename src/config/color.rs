use core::fmt;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

use anyhow::Result;
use crossterm::tty::IsTty;
use ratatui::style::Styled;
use serde::{Deserialize, Serialize};

use super::Switch;
use crate::libnala::Operation;
use crate::logger::Level;

pub type RatStyle = ratatui::style::Style;
pub type RatColor = ratatui::style::Color;
pub type RatMod = ratatui::style::Modifier;

/// Default Modifier for Serde
fn bold() -> RatMod { RatMod::BOLD }

static REF: OnceLock<Color> = OnceLock::new();

pub fn get() -> &'static Color { REF.get_or_init(Color::default) }
pub fn _set(color: Color) -> Result<(), Color> { REF.set(color) }

#[macro_export]
macro_rules! color {
	($theme:expr, $string:expr) => {{
		$crate::config::color::get().color($theme, &$string)
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

pub use {color, highlight, primary, secondary, ver};

#[derive(Debug, Serialize, Deserialize)]
pub struct Color {
	pub level: Level,
	pub switch: Switch,
	pub map: HashMap<Theme, Style>,
}

unsafe impl Sync for Color {}

impl Default for Color {
	fn default() -> Self { Color::new(Level::Notice, Switch::Auto, Theme::style_map()) }
}

impl Color {
	pub fn new(level: Level, switch: Switch, map: HashMap<Theme, Style>) -> Color {
		Color { level, switch, map }
	}

	pub fn from_config() -> Color {
		let Ok(str) = std::fs::read_to_string(Path::new("/etc/nala/theme.conf")) else {
			return Color::default();
		};
		toml::from_str(&str).unwrap_or_default()
	}

	pub fn can_color(&self) -> bool {
		match self.switch {
			Switch::Always => true,
			Switch::Never => false,
			Switch::Auto => std::io::stdout().is_tty(),
		}
	}

	pub fn color<'a, D: AsRef<str> + ?Sized>(&self, theme: Theme, string: &'a D) -> Cow<'a, str> {
		let string = string.as_ref();

		if self.can_color() {
			if let Some(style) = self.map.get(&theme) {
				return Cow::Owned(format!("{style}{string}\x1b[0m"));
			}
		}

		Cow::Borrowed(string)
	}

	pub fn rat_style<T: AsRef<Theme>>(&self, theme: T) -> RatStyle {
		self.map
			.get(theme.as_ref())
			.unwrap_or(&Style::default())
			.to_rat()
	}

	pub fn rat_reset<T: AsRef<Theme>>(&self, theme: T) -> RatStyle {
		RatStyle::reset().set_style(self.rat_style(theme))
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

impl From<Operation> for Theme {
	fn from(value: Operation) -> Theme {
		match value {
			Operation::Remove | Operation::AutoRemove | Operation::Purge | Operation::AutoPurge => {
				Theme::Error
			},
			Operation::Install | Operation::Upgrade => Theme::Secondary,
			Operation::Reinstall | Operation::Downgrade | Operation::Held => Theme::Notice,
		}
	}
}

impl Theme {
	pub fn style_map() -> HashMap<Theme, Style> {
		let mut map = HashMap::new();
		for theme in Theme::iter() {
			map.insert(*theme, theme.default_style());
		}
		map
	}

	pub fn iter() -> &'static [Theme] {
		&[
			Theme::Primary,
			Theme::Secondary,
			Theme::Regular,
			Theme::Highlight,
			Theme::ProgressFilled,
			Theme::ProgressUnfilled,
			Theme::Notice,
			Theme::Warning,
			Theme::Error,
		]
	}

	pub fn default_style(&self) -> Style {
		match self {
			Theme::Primary => Style::bold(RatColor::LightGreen),
			Theme::Secondary => Style::bold(RatColor::LightBlue),
			Theme::Regular => Style::no_bold(RatColor::White),
			Theme::Highlight => Style::bold(RatColor::White),

			Theme::ProgressFilled => Style::bold(RatColor::LightGreen),
			Theme::ProgressUnfilled => Style::bold(RatColor::LightRed),

			Theme::Notice => Style::bold(RatColor::LightYellow),
			Theme::Warning => Style::bold(RatColor::LightYellow),
			Theme::Error => Style::bold(RatColor::LightRed),
		}
	}
}

impl AsRef<Theme> for Theme {
	fn as_ref(&self) -> &Theme { self }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Style {
	fg: RatColor,
	bg: Option<RatColor>,
	#[serde(default = "bold")]
	modifier: RatMod,
	/// ANSI Code that goes before the string.
	#[serde(skip)]
	string: OnceLock<String>,
}

impl Style {
	pub fn new(modifier: RatMod, fg: RatColor, bg: Option<RatColor>) -> Self {
		Self {
			fg,
			bg,
			modifier,
			string: OnceLock::new(),
		}
	}

	pub fn default() -> Self { Self::no_bold(RatColor::White) }

	pub fn bold(color: RatColor) -> Self { Self::new(RatMod::BOLD, color, None) }

	pub fn no_bold(color: RatColor) -> Self { Self::new(RatMod::empty(), color, None) }

	pub fn to_rat(&self) -> RatStyle {
		let rat = RatStyle::default().fg(self.fg).add_modifier(self.modifier);

		if let Some(bg) = self.bg {
			return rat.bg(bg);
		}
		rat
	}

	pub fn ansi_color(&self) -> &str {
		match self.fg {
			RatColor::Reset => "0",
			RatColor::Black => "30",
			RatColor::Red => "31",
			RatColor::Green => "32",
			RatColor::Yellow => "33",
			RatColor::Blue => "34",
			RatColor::Magenta => "35",
			RatColor::Cyan => "36",
			RatColor::Gray => "37",
			RatColor::DarkGray => "90",
			RatColor::LightRed => "91",
			RatColor::LightGreen => "92",
			RatColor::LightYellow => "93",
			RatColor::LightBlue => "94",
			RatColor::LightMagenta => "95",
			RatColor::LightCyan => "96",
			RatColor::White => "97",
			_ => unreachable!(),
		}
	}

	pub fn mod_string(&self) -> String {
		[
			(RatMod::BOLD, "1"),
			(RatMod::DIM, "2"),
			(RatMod::ITALIC, "3"),
			(RatMod::UNDERLINED, "4"),
			(RatMod::SLOW_BLINK, "5"),
			(RatMod::RAPID_BLINK, "6"),
			(RatMod::REVERSED, "7"),
			(RatMod::HIDDEN, "8"),
			(RatMod::CROSSED_OUT, "9"),
		]
		.into_iter()
		.filter_map(|(m, a)| self.modifier.contains(m).then_some(a))
		.collect::<Vec<&str>>()
		.join(";")
	}
}

impl fmt::Display for Style {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let string = self.string.get_or_init(|| {
			let ansi_color = match self.fg {
				RatColor::Rgb(r, g, b) => &format!("38;2;{r};{g};{b}"),
				RatColor::Indexed(int) => &format!("38;5;{int}"),
				_ => self.ansi_color(),
			};
			format!("\x1b[{};{ansi_color}m", self.mod_string())
		});
		write!(f, "{string}")
	}
}
