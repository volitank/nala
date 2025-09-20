use std::io::{stdout, Write};
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use anyhow::Result;
use crossterm::terminal::disable_raw_mode;
use indexmap::IndexMap;
use indicatif::ProgressBar;
use ratatui::backend::Backend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{LineGauge, Paragraph, Widget, Wrap};
use ratatui::{symbols, Frame};
use regex::Regex;
use rust_apt::util::time_str;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use super::{Drawable, Term};
use crate::config::{Config, Theme};
use crate::deb::DebFile;
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

#[derive(Clone)]
pub struct DisplayGroup<'a> {
	title: Option<String>,
	_value: Option<&'a dyn Widget>,
	map: IndexMap<String, String>,
}

impl<'a> DisplayGroup<'a> {
	pub fn new(title: Option<String>, _value: Option<&'a dyn Widget>) -> DisplayGroup<'a> {
		Self {
			title,
			_value,
			map: IndexMap::new(),
		}
	}

	pub fn new_no_value(title: &str) -> Self { Self::new(Some(title.to_string()), None) }

	pub fn _new_value(title: String, value: &'a dyn Widget) -> DisplayGroup<'a> {
		Self::new(Some(title), Some(value))
	}

	/// Prints just a single string
	pub fn _single_string(header: &str, msg: String) -> DisplayGroup<'_> {
		// TODO: Now that we have value field, I think this can be different
		let mut dg = DisplayGroup::default();
		dg.insert(header.to_string(), msg);
		dg
	}
}

impl Default for DisplayGroup<'_> {
	fn default() -> Self { Self::new(None, None) }
}

impl Deref for DisplayGroup<'_> {
	type Target = IndexMap<String, String>;

	fn deref(&self) -> &Self::Target { &self.map }
}

impl DerefMut for DisplayGroup<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.map }
}

impl Drawable for DisplayGroup<'_> {
	fn draw(&self, config: &Config, f: &mut Frame, area: Rect) {
		let inner = if let Some(title) = self.title.as_ref() {
			borderless_area(f, area, title)
		} else {
			area
		};

		let stop = self.map.len();
		let mut con: Vec<Constraint> = (0..stop).map(|_| Constraint::Length(1)).collect();
		con.push(Constraint::Min(0));

		let info = Layout::vertical(con).split(inner);

		// Plus 1 is for the space between columns
		let max_header = self.map.keys().map(|s| s.len()).max().unwrap_or(6) + 1;

		for (i, (key, value)) in self.iter().enumerate() {
			let mut line = Line::default();
			line.push_span(Span::from(key).style(config.color.rat_reset(Theme::Primary)));
			line.push_span(Span::raw(" ".repeat(max_header - key.len())));
			line.push_span(Span::from(value).style(config.color.rat_reset(Theme::Regular)));

			Paragraph::new(line)
				.wrap(Wrap { trim: false })
				.render(info[i], f.buffer_mut());
		}
	}

	fn height(&self) -> u16 { self.map.len() as u16 }
}

impl Deref for NalaProgressBar {
	type Target = ProgressBar;

	fn deref(&self) -> &Self::Target { &self.pb }
}

pub struct NalaProgressBar {
	pub pb: ProgressBar,
	pub unit: UnitStr,
	ansi: Regex,
	pub disabled: bool,
	pub extra_info: Vec<(String, String)>,
}

impl NalaProgressBar {
	pub fn new() -> Result<Self> {
		let pb = ProgressBar::hidden();
		pb.set_length(0);

		let ret = Self {
			pb,
			unit: UnitStr::new(1, NumSys::Binary),
			ansi: Regex::new(r"\x1b\[([\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e])")?,
			disabled: false,
			extra_info: vec![],
		};

		Ok(ret)
	}

	pub fn set_info(&mut self, info: Vec<(String, String)>) { self.extra_info = info; }

	pub async fn join(
		&mut self,
		config: &Config,
		term: &mut Term,
		mut set: JoinSet<Result<DebFile>>,
	) -> Result<Vec<DebFile>> {
		self.pb.set_length(set.len() as u64);
		let mut ret = vec![];

		while let Some(res) = set.join_next().await {
			let item = res??;
			self.inc(1);

			term.term.draw(|f| self.draw(config, f, f.area()))?;
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
	pub fn elapsed(&self) -> u64 { self.pb.elapsed().as_secs_f64().ceil() as u64 }

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

	pub fn bar(&self, config: &Config) -> LineGauge<'_> {
		LineGauge::default()
			.line_set(symbols::line::THICK)
			.ratio(self.ratio())
			.label(self.label())
			.filled_style(config.color.rat_style(Theme::ProgressFilled))
			.unfilled_style(config.color.rat_style(Theme::ProgressUnfilled))
	}

	pub fn print(&mut self, config: &Config, term: &mut Term, msg: &str) -> Result<()> {
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
				.style(config.color.rat_style(Theme::Regular))
				.render(buf.area, buf);
		})?;
		// Must redraw the terminal after printing
		term.draw(config, &[self])?;
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
	pub fn label(&self) -> Line<'_> { Line::from("Progress:") }

	pub fn current_total(&self) -> String {
		format!(
			"{}/{}",
			self.unit.str(self.pb.position()),
			self.unit.str(self.length()),
		)
	}
}

pub fn split_horizontal(area: Rect) -> Rc<[Rect]> {
	Layout::horizontal([Constraint::Max(32), Constraint::Max(32), Constraint::Min(0)]).split(area)
}

impl Drawable for NalaProgressBar {
	fn draw(&self, config: &Config, f: &mut Frame, area: Rect) {
		let [bar_area, info_area] =
			Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);

		let pb = self.bar(config);
		// Draw Progress Bar
		let half_bar = split_horizontal(bar_area);
		pb.render(half_bar[0], f.buffer_mut());

		let dg_area = split_horizontal(info_area);

		let mut dg1 = DisplayGroup::default();
		let mut dg2 = DisplayGroup::default();

		for (dg, items) in [
			(
				&mut dg1,
				vec![
					("Total", self.current_total()),
					(
						"Speed",
						format!("{}/s", self.unit.str(self.per_sec() as u64)),
					),
				],
			),
			(
				&mut dg2,
				vec![
					("Elapsed", time_str(self.elapsed())),
					("Remaining", time_str(self.pb.eta().as_secs())),
				],
			),
		] {
			for (k, v) in items {
				dg.insert(format!("  {k}:"), v);
			}
		}

		for (i, (k, v)) in self.extra_info.clone().into_iter().enumerate() {
			if i % 2 == 0 {
				dg1.insert(format!("  {k}:"), v);
			} else {
				dg2.insert(format!("  {k}:"), v);
			}
		}

		dg1.draw(config, f, dg_area[0]);
		dg2.draw(config, f, dg_area[1]);
	}

	fn height(&self) -> u16 { 4 }
}
