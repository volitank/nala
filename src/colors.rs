use core::fmt;
use std::cell::OnceCell;

use serde::Deserialize;

pub type RatStyle = ratatui::style::Style;
pub type RatColor = ratatui::style::Color;
pub type RatMod = ratatui::style::Modifier;

/// Default Modifier for Serde
fn bold() -> RatMod { RatMod::BOLD }

#[derive(Deserialize, Debug, Hash, Eq, PartialEq, Copy, Clone)]
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

impl Theme {
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

#[derive(Deserialize, Debug)]
pub struct Style {
	fg: RatColor,
	bg: Option<RatColor>,
	#[serde(default = "bold")]
	modifier: RatMod,
	/// ANSI Code that goes before the string.
	#[serde(skip)]
	string: OnceCell<String>,
}

impl Style {
	pub fn new(modifier: RatMod, fg: RatColor, bg: Option<RatColor>) -> Self {
		Self {
			fg,
			bg,
			modifier,
			string: OnceCell::new(),
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
