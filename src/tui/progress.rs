use std::io::{stdout, Write};
use std::ops::{Deref, DerefMut};

use anyhow::Result;
use crossterm::terminal::disable_raw_mode;
use indexmap::IndexMap;
use indicatif::ProgressBar;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::symbols::{block, border};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget, Wrap};
use ratatui::{symbols, Frame};
use regex::Regex;
use rust_apt::util::time_str;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use super::{Drawable, Term};
use crate::config::{Config, Theme};
use crate::tui::{self, borderless_area};

/// Numeral System for unit conversion.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum NumSys {
	/// Base 2 | 1024 | KibiByte (KiB)
	Binary,
	/// Base 10 | 1000 | KiloByte (KB)
	Decimal,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct UnitStr {
	#[serde(default)]
	precision: usize,
	base: NumSys,
}

impl UnitStr {
	pub fn new(precision: usize, base: NumSys) -> UnitStr { UnitStr { precision, base } }

	pub fn str(&self, val: u64) -> String {
		let val = val as f64;
		let (num, tera, giga, mega, kilo) = match self.base {
			NumSys::Binary => (1024.0_f64, "TiB", "GiB", "MiB", "KiB"),
			NumSys::Decimal => (1000.0_f64, "TB", "GB", "MB", "KB"),
		};

		let powers = [
			(num.powi(4), tera),
			(num.powi(3), giga),
			(num.powi(2), mega),
			(num, kilo),
		];

		for (divisor, unit) in powers {
			if val > divisor {
				return format!("{:.1$} {unit}", val / divisor, self.precision);
			}
		}
		format!("{val} B")
	}
}

pub trait ProgressItem {
	fn header(&self) -> String;
	fn msg(&self) -> String;
}

// impl<'a> Widget for &PkgProgress<'a> {
// 	fn render(self, area: Rect, buf: &mut Buffer) {
// 		let mut line = Line::default();
// 		line.push_span(Span::from(&self.header).style(self.config.color.rat_reset(Theme::Primary)));

// 		line.push_span(Span::raw(" "));

// 		for msg in &self.lines {
// 			line.push_span(Span::from(msg).style(self.config.color.rat_reset(Theme::Regular)));
// 		}

// 		Paragraph::new(line).wrap(Wrap { trim: false }).render(area, buf);
// 	}
// }

#[derive(Clone, Debug)]
pub struct DisplayGroup<'a> {
	config: &'a Config,
	title: Option<String>,
	map: IndexMap<String, String>,
}

impl<'a> DisplayGroup<'a> {
	pub fn new(config: &'a Config, title: Option<String>) -> DisplayGroup<'a> {
		Self {
			config,
			title,
			map: IndexMap::new(),
		}
	}

	pub fn new_str(config: &'a Config, title: &str) -> Self {
		Self::new(config, Some(title.to_string()))
	}
}

impl<'a> Deref for DisplayGroup<'a> {
	type Target = IndexMap<String, String>;

	fn deref(&self) -> &Self::Target { &self.map }
}


impl<'a> DerefMut for DisplayGroup<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.map
	}
}

impl<'a> Drawable for DisplayGroup<'a> {
	fn draw(&self, f: &mut Frame, area: Rect) {
		let inner = if let Some(title) = self.title.as_ref() {
			borderless_area(f, area, title)
		} else {
			area
		};

		let stop = self.map.len();
		let mut con: Vec<Constraint> = (0..stop).map(|_| Constraint::Length(1)).collect();
		// con.push(Constraint::Min(0));

		let info = Layout::vertical(con).split(inner);

		let col_size = self.map.keys().map(|s| s.len()).max().unwrap_or(6);

		for (i, (key, value)) in self.iter().enumerate() {
			if i >= stop {
				break;
			}

			let mut line = Line::default();
			line.push_span(Span::from(key).style(self.config.color.rat_reset(Theme::Primary)));
			line.push_span(Span::raw(" "));
			line.push_span(Span::from(value).style(self.config.color.rat_reset(Theme::Regular)));

			Paragraph::new(line).wrap(Wrap { trim: false }).render(info[i], f.buffer_mut());
		}
	}
}

impl Deref for NalaProgressBar<'_> {
	type Target = ProgressBar;

	fn deref(&self) -> &Self::Target { &self.pb }
}

pub struct NalaProgressBar<'a> {
	pub config: &'a Config,
	pub pb: ProgressBar,
	pub unit: UnitStr,
	ansi: Regex,
	pub disabled: bool,
}

impl<'a> NalaProgressBar<'a> {
	pub fn new(config: &'a Config) -> Result<Self> {
		let pb = ProgressBar::hidden();
		pb.set_length(0);

		let ret = Self {
			config,
			pb,
			unit: UnitStr::new(1, NumSys::Binary),
			ansi: Regex::new(r"\x1b\[([\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e])")?,
			disabled: false,
		};

		Ok(ret)
	}

	pub async fn join<P: ProgressItem + 'static>(
		&mut self,
		term: &mut Term,
		mut set: JoinSet<Result<P>>,
	) -> Result<Vec<P>> {
		self.pb.set_length(set.len() as u64);
		let mut ret = vec![];
		while let Some(res) = set.join_next().await {
			let item = res??;
			// self.dg.push_str(item.header(), item.msg());
			self.inc(1);

			term.term.draw(|f| self.draw(f, f.area()))?;
			if tui::poll_exit_event()? {
				self.clean_up(term)?;
				std::process::exit(1);
			}
			ret.push(item);
		}

		self.clean_up(term)?;
		Ok(ret)
	}

	pub fn length(&self) -> u64 { self.pb.length().unwrap_or_default() }

	// f64 as ceil incase it's less than 1 second we round up to that.
	fn elapsed(&self) -> u64 { self.pb.elapsed().as_secs_f64().ceil() as u64 }

	pub fn ratio(&self) -> f64 {
		let ratio = self.pb.position() as f64 / self.length() as f64;
		if ratio > 1.0 {
			return 1.0;
		}
		ratio
	}

	pub fn hidden(&self) -> bool { self.disabled }

	pub fn hide(&mut self, term: &mut Term) -> Result<()> {
		term.clear()?;
		term.show_cursor()?;
		self.disabled = true;
		Ok(())
	}

	pub fn unhide(&mut self, term: &mut Term) -> Result<()> {
		writeln!(stdout(), "\n\n\n")?;
		term.hide_cursor()?;
		self.disabled = false;
		Ok(())
	}

	pub fn clean_up(&mut self, term: &mut Term) -> Result<()> {
		term.clear()?;
		disable_raw_mode()?;
		term.show_cursor()?;
		Ok(())
	}

	pub fn bar(&self) -> LineGauge<'_> {
		LineGauge::default()
			.line_set(symbols::line::THICK)
			.ratio(self.ratio())
			.label(self.label())
			.filled_style(self.config.color.rat_style(Theme::ProgressFilled))
			.unfilled_style(self.config.color.rat_style(Theme::ProgressUnfilled))
	}

	pub fn print(&mut self, term: &mut Term, msg: &str) -> Result<()> {
		if self.disabled {
			return Ok(());
		}

		// Strip ansi escape codes to get the correct size of the message
		let height =
			self.ansi.replace_all(msg, "").len() as f32 / term.backend().size()?.width as f32;

		// Check how many new lines as well
		let lines = (height.ceil() as u16).max(msg.lines().count() as u16);

		// Artifacts come into play if the viewport isn't cleared
		term.clear()?;
		term.insert_before(lines, |buf| {
			Paragraph::new(msg)
				.left_aligned()
				.wrap(Wrap::default())
				.style(self.config.color.rat_style(Theme::Regular))
				.render(buf.area, buf);
		})?;
		// Must redraw the terminal after printing
		term.draw(self.config, &[self])?;
		Ok(())
	}

	pub fn finished_string(&self) -> String {
		// I've seen this erroneously as 1 before.
		if self.length() > 1 {
			format!(
				"Fetched {} in {} ({}/s)",
				self.unit.str(self.length()),
				time_str(self.elapsed()),
				self.unit.str(self.length() / self.elapsed())
			)
		} else {
			"Nothing to fetch".to_string()
		}
	}

	/// TODO: Turn this into a trait!!!
	pub fn label(&self) -> Line<'_> {
		Line::from(rust_apt::util::time_str(self.pb.eta().as_secs())).style(
			self.config.color.rat_style(Theme::Regular)
		)
	}

	pub fn current_total(&self) -> String {
		format!(
			"{}/{}",
			self.unit.str(self.pb.position()),
			self.unit.str(self.length()),
		)
	}
}
