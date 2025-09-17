use std::io::{stdout, Write};
use std::ops::Deref;

use anyhow::Result;
use crossterm::terminal::disable_raw_mode;
use indexmap::IndexMap;
use indicatif::ProgressBar;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, LineGauge, Paragraph, Widget, Wrap};
use ratatui::{symbols, Frame};
use regex::Regex;
use rust_apt::util::time_str;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

use super::{Drawable, Term};
use crate::config::{Config, Theme};
use crate::{color, tui};

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

// #[derive(Debug)]
// pub struct Progress<'a> {
// 	dpkg: bool,
// 	percentage: String,
// 	current_total: String,
// 	per_sec: String,
// 	bar: LineGauge<'a>,
// 	spans: DisplayGroup,
// 	themes: (Style, Style),
// }

// impl Widget for Progress<'_> {
// 	fn render(mut self, area: Rect, buf: &mut Buffer) {
// 		let block = Block::bordered()
// 			.border_type(BorderType::Rounded)
// 			.padding(Padding::horizontal(1))
// 			.style(self.themes.0);

// 		let [remainder, bar] =
// 			Layout::vertical([Constraint::Min(0),
// Constraint::Length(1)]).areas(block.inner(area));

// 		let mut constraints = vec![
// 			Constraint::Min(0),
// 			Constraint::Length(self.percentage.len() as u16 + 2),
// 			Constraint::Length(self.current_total.len() as u16 + 2),
// 		];

// 		let bar_block = if self.dpkg {
// 			Layout::horizontal(constraints).split(block.inner(area))
// 		} else {
// 			constraints.push(Constraint::Length(self.per_sec.len() as u16 + 2));
// 			Layout::horizontal(constraints)
// 				.flex(Flex::SpaceBetween)
// 				.split(bar)
// 		};

// 		block.render(area, buf);
// 		if !self.dpkg {
// 			self.spans.render(remainder, buf);

// 			paragraph(&self.per_sec)
// 				.style(self.themes.1)
// 				.render(bar_block[2], buf);
// 		}

// 		self.bar.render(bar_block[0], buf);
// 		paragraph(&self.percentage)
// 			.style(self.themes.1)
// 			.render(bar_block[0], buf);

// 		if !self.dpkg {
// 			paragraph(&self.current_total)
// 				.style(self.themes.0)
// 				.render(bar_block[2], buf);
// 		}
// 	}
// }

#[derive(Clone, Debug)]
pub struct PkgProgress<'a> {
	config: &'a Config,
	header: String,
	theme: Theme,
	lines: Vec<String>,
}

impl<'a> PkgProgress<'a> {
	pub fn new(config: &'a Config, header: String) -> PkgProgress<'a> {
		Self {
			config,
			header,
			theme: Theme::Primary,
			lines: vec![],
		}
	}

	pub fn lite_clone(&self) -> PkgProgress<'a> {
		PkgProgress::new(self.config, self.header.to_string())
	}

	pub fn theme(mut self, theme: Theme) -> Self {
		self.theme = theme;
		self
	}

	pub fn add_msg(&mut self, value: String) { self.lines.push(value) }

	pub fn into_line(self) -> Line<'static> {
		let mut line = Line::default();
		line.push_span(Span::from(self.header).style(self.config.color.rat_reset(Theme::Primary)));

		for msg in self.lines {
			line.push_span(Span::from(msg).style(self.config.color.rat_reset(Theme::Regular)));
		}
		line
	}
}

impl<'a> Widget for &PkgProgress<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let mut line = Line::default();
		line.push_span(Span::from(&self.header).style(self.config.color.rat_reset(Theme::Primary)));

		for msg in &self.lines {
			line.push_span(Span::from(msg).style(self.config.color.rat_reset(Theme::Regular)));
		}

		Paragraph::new(line).render(area, buf);
	}
}

#[derive(Clone, Debug)]
pub struct DisplayGroup<'a> {
	config: &'a Config,
	map: IndexMap<String, PkgProgress<'a>>,
}

impl<'a> DisplayGroup<'a> {
	pub fn new(config: &'a Config) -> DisplayGroup<'a> {
		Self {
			config,
			map: IndexMap::new(),
		}
	}

	pub fn push(&mut self, value: PkgProgress<'a>) -> &mut Self {
		self.map.insert(value.header.clone(), value);
		self
	}

	pub fn push_str(&mut self, key: String, value: String) -> &mut Self {
		if let Some(pkg) = self.map.get_mut(&key) {
			pkg.lines = vec![value];
			return self;
		}

		let mut pkg = PkgProgress::new(self.config, key);
		pkg.add_msg(value);
		self.push(pkg);
		self
	}
}

impl<'a> Deref for DisplayGroup<'a> {
	type Target = IndexMap<String, PkgProgress<'a>>;

	fn deref(&self) -> &Self::Target { &self.map }
}

impl<'a> Widget for &mut DisplayGroup<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let stop = self.map.len();

		let mut con: Vec<Constraint> = (0..stop).map(|_| Constraint::Length(1)).collect();
		con.push(Constraint::Min(0));

		let inner = Layout::vertical(con).flex(Flex::Center).split(area);

		for (i, pkg) in self.values().enumerate() {
			if i >= stop {
				break;
			}
			pkg.render(inner[i], buf);
		}
	}
}

impl<'a> Drawable for DisplayGroup<'a> {
	fn draw(&self, f: &mut Frame, area: Rect) {
		let stop = self.map.len();

		let mut con: Vec<Constraint> = (0..stop).map(|_| Constraint::Length(1)).collect();
		con.push(Constraint::Min(0));

		let inner = Layout::vertical(con).flex(Flex::Center).split(area);

		for (i, pkg) in self.values().enumerate() {
			if i >= stop {
				break;
			}
			pkg.render(inner[i], f.buffer_mut());
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
			.label(self.label().into_line())
			.filled_style(self.config.color.rat_style(Theme::ProgressFilled))
			.unfilled_style(self.config.color.rat_style(Theme::ProgressUnfilled))
	}

	pub fn constraints(&self, block: &Block<'static>, area: Rect) -> [Rect; 2] {
		Layout::horizontal([Constraint::Fill(100), Constraint::Min(6)]).areas(block.inner(area))
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
	pub fn label(&self) -> PkgProgress {
		let mut msg = PkgProgress::new(self.config, "Remaining: ".to_string());
		if self.pb.position() < self.length() {
			msg.add_msg(rust_apt::util::time_str(self.pb.eta().as_secs()));
		}
		msg
	}

	pub fn current_total(&self) -> String {
		format!(
			" {}/{}",
			self.unit.str(self.pb.position()),
			self.unit.str(self.length()),
		)
	}
}

impl Widget for &NalaProgressBar<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		if self.disabled {
			return;
		}
		let block = tui::vblock(&self.config.color);
		let prog_bar = self.bar();

		let [bar_area, percent_area] = self.constraints(&block, area);

		block.render(area, buf);

		prog_bar.render(bar_area, buf);

		let percent = format!(" {:.1}%", self.ratio() * 100.0);
		tui::paragraph(&percent)
			.style(self.config.color.rat_style(Theme::Primary))
			.render(percent_area, buf);
	}
}
