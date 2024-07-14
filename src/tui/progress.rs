use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use indicatif::ProgressBar;
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Style, Styled};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, LineGauge, Padding, Paragraph, Widget, Wrap};
use ratatui::{symbols, Frame, Terminal, TerminalOptions, Viewport};
use regex::Regex;
use rust_apt::util::{time_str, NumSys};

use crate::colors::Theme;
use crate::config::Config;

pub struct UnitStr {
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

pub struct NalaProgressBar<'a> {
	terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
	config: &'a Config,
	pub indicatif: ProgressBar,
	pub unit: UnitStr,
	pub msg: Vec<String>,
	ansi: Regex,
}

impl<'a> NalaProgressBar<'a> {
	pub fn new(config: &'a Config) -> Result<Self> {
		let indicatif = ProgressBar::hidden();
		indicatif.set_length(0);

		enable_raw_mode()?;
		let terminal = Terminal::with_options(
			CrosstermBackend::new(std::io::stdout()),
			TerminalOptions {
				viewport: Viewport::Inline(4),
			},
		)?;

		Ok(Self {
			terminal,
			config,
			indicatif,
			unit: UnitStr::new(1, NumSys::Binary),
			msg: vec![],
			ansi: Regex::new(r"\x1b\[([\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e])")?,
		})
	}

	pub fn length(&self) -> u64 { self.indicatif.length().unwrap_or_default() }

	// f64 as ceil incase it's less than 1 second we round up to that.
	fn elapsed(&self) -> u64 { self.indicatif.elapsed().as_secs_f64().ceil() as u64 }

	fn ratio(&self) -> f64 { self.indicatif.position() as f64 / self.length() as f64 }

	pub fn clean_up(&mut self) -> Result<()> {
		self.terminal.clear()?;
		disable_raw_mode()?;
		self.terminal.show_cursor()?;
		Ok(())
	}

	pub fn print(&mut self, msg: String) -> Result<()> {
		// Strip ansi escape codes to get the correct size of the message
		let height = self.ansi.replace_all(&msg, "").len() as f32
			/ self.terminal.backend().size()?.width as f32;

		self.terminal.insert_before(height.ceil() as u16, |buf| {
			Paragraph::new(msg)
				.left_aligned()
				.wrap(Wrap::default())
				.style(self.config.rat_style(Theme::Regular))
				.render(buf.area, buf);
		})?;
		// Must redraw the terminal after printing
		self.render()
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

	pub fn span(&self, theme: Theme, string: &'a str) -> Span<'a> {
		let style = Style::reset().set_style(self.config.rat_style(theme));
		Span::from(string).style(style)
	}

	pub fn render(&mut self) -> Result<()> {
		let mut spans = vec![];

		if self.msg.is_empty() {
			spans.push(self.span(Theme::Primary, "Working..."))
		} else {
			let mut header = true;
			for string in self.msg.iter() {
				if header {
					spans.push(self.span(Theme::Primary, string));
					header = false;
					continue;
				}
				spans.push(self.span(Theme::Regular, string));
				header = true;
			}
		}

		let percentage = format!("{:.1}%", self.ratio() * 100.0);
		let current_total = format!(
			"{}/{}",
			self.unit.str(self.indicatif.position()),
			self.unit.str(self.length()),
		);
		let per_sec = format!("{}/s", self.unit.str(self.indicatif.per_sec() as u64));
		let eta = format!(
			" {}",
			rust_apt::util::time_str(self.indicatif.eta().as_secs())
		);

		let label = if self.indicatif.position() < self.length() {
			vec![
				self.span(Theme::Primary, "Time Remaining:"),
				self.span(Theme::Regular, &eta),
			]
		} else {
			vec![self.span(Theme::Primary, "Working...")]
		};

		let bar = LineGauge::default()
			.line_set(symbols::line::THICK)
			.ratio(self.ratio())
			.label(Line::from(label))
			.filled_style(self.config.rat_style(Theme::ProgressFilled))
			.unfilled_style(self.config.rat_style(Theme::ProgressUnfilled));

		let themes = (
			self.config.rat_style(Theme::Primary),
			self.config.rat_style(Theme::Secondary),
		);

		self.terminal
			.draw(|f| render(f, bar, percentage, current_total, per_sec, spans, themes))?;

		Ok(())
	}
}

pub fn render(
	f: &mut Frame,
	bar: LineGauge,
	percentage: String,
	current_total: String,
	per_sec: String,
	spans: Vec<Span>,
	themes: (Style, Style),
) {
	let block = Block::bordered()
		.border_type(BorderType::Rounded)
		.padding(Padding::horizontal(1))
		.style(themes.0);

	let inner = Layout::vertical([Constraint::Length(1), Constraint::Length(1)])
		.split(block.inner(f.size()));

	let bar_block = Layout::horizontal([
		Constraint::Fill(100),
		Constraint::Length(percentage.len() as u16 + 2),
		Constraint::Length(current_total.len() as u16 + 2),
		Constraint::Length(per_sec.len() as u16 + 2),
	])
	.split(inner[1]);

	f.render_widget(block, f.size());
	f.render_widget(Paragraph::new(Line::from(spans)), inner[0]);

	f.render_widget(bar, bar_block[0]);
	f.render_widget(get_paragraph(&percentage).style(themes.1), bar_block[1]);
	f.render_widget(get_paragraph(&current_total).style(themes.0), bar_block[2]);
	f.render_widget(get_paragraph(&per_sec).style(themes.1), bar_block[3]);
}

pub fn get_paragraph(text: &str) -> Paragraph { Paragraph::new(text).right_aligned() }
