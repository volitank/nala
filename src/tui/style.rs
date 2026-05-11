use ratatui::style::{Color as RatColor, Modifier as RatMod, Style as RatStyle};

use crate::config::color::{ColorCode, Modifiers, Style, Theme};
use crate::config::{Config, Switch};

fn to_rat_color(color: &ColorCode) -> RatColor {
	match color {
		ColorCode::Reset => RatColor::Reset,
		ColorCode::Black => RatColor::Black,
		ColorCode::Red => RatColor::Red,
		ColorCode::Green => RatColor::Green,
		ColorCode::Yellow => RatColor::Yellow,
		ColorCode::Blue => RatColor::Blue,
		ColorCode::Magenta => RatColor::Magenta,
		ColorCode::Cyan => RatColor::Cyan,
		ColorCode::Gray => RatColor::Gray,
		ColorCode::DarkGray => RatColor::DarkGray,
		ColorCode::LightRed => RatColor::LightRed,
		ColorCode::LightGreen => RatColor::LightGreen,
		ColorCode::LightYellow => RatColor::LightYellow,
		ColorCode::LightBlue => RatColor::LightBlue,
		ColorCode::LightMagenta => RatColor::LightMagenta,
		ColorCode::LightCyan => RatColor::LightCyan,
		ColorCode::White => RatColor::White,
		ColorCode::Indexed(i) => RatColor::Indexed(*i),
		ColorCode::Rgb(r, g, b) => RatColor::Rgb(*r, *g, *b),
	}
}

fn to_rat_modifiers(mods: Modifiers) -> RatMod {
	let mut rat = RatMod::empty();
	let mapping = [
		(Modifiers::BOLD, RatMod::BOLD),
		(Modifiers::DIM, RatMod::DIM),
		(Modifiers::ITALIC, RatMod::ITALIC),
		(Modifiers::UNDERLINED, RatMod::UNDERLINED),
		(Modifiers::SLOW_BLINK, RatMod::SLOW_BLINK),
		(Modifiers::RAPID_BLINK, RatMod::RAPID_BLINK),
		(Modifiers::REVERSED, RatMod::REVERSED),
		(Modifiers::HIDDEN, RatMod::HIDDEN),
		(Modifiers::CROSSED_OUT, RatMod::CROSSED_OUT),
	];

	for (modifier, rat_modifier) in mapping {
		if mods.contains(modifier) {
			rat.insert(rat_modifier);
		}
	}

	rat
}

fn to_rat_style(config: &Config, style: &Style) -> RatStyle {
	if config.color_mode() == Switch::Never {
		return RatStyle::default();
	}

	let mut rat = RatStyle::default()
		.fg(to_rat_color(&style.fg))
		.add_modifier(to_rat_modifiers(style.modifier));

	if let Some(bg) = style.bg {
		rat = rat.bg(to_rat_color(&bg));
	}

	rat
}

pub fn style(config: &Config, theme: impl AsRef<Theme>) -> RatStyle {
	to_rat_style(config, config.style(theme))
}

pub fn reset(config: &Config, theme: impl AsRef<Theme>) -> RatStyle {
	RatStyle::reset().patch(style(config, theme))
}
