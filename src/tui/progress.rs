use std::io::{stdout, Write};

use anyhow::Result;
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, LineGauge, Padding, Paragraph, Widget, Wrap};
use regex::Regex;
use rust_apt::util::time_str;

use super::style as tui_style;
use crate::config::{Config, Theme};
use crate::progress::{DisplayGroup, ProgressMessage, ProgressState};
use crate::terminal::Term;

#[derive(Debug)]
struct ProgressWidget<'a> {
	dpkg: bool,
	percentage: String,
	current_total: String,
	per_sec: String,
	bar: LineGauge<'a>,
	spans: Vec<Line<'a>>,
	themes: (Style, Style),
}

impl Widget for ProgressWidget<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_type(BorderType::Rounded)
			.padding(Padding::horizontal(1))
			.style(self.themes.0);

		let inner = Layout::vertical([Constraint::Fill(100), Constraint::Length(1)])
			.split(block.inner(*buf.area()));

		let mut constraints = vec![
			Constraint::Fill(100),
			Constraint::Length(self.percentage.len() as u16 + 2),
			Constraint::Length(self.current_total.len() as u16 + 2),
		];

		let bar_block = if self.dpkg {
			Layout::horizontal(constraints).split(block.inner(*buf.area()))
		} else {
			constraints.push(Constraint::Length(self.per_sec.len() as u16 + 2));
			Layout::horizontal(constraints).split(inner[1])
		};

		block.render(area, buf);
		if !self.dpkg {
			Paragraph::new(self.spans).render(inner[0], buf);

			get_paragraph(&self.per_sec)
				.style(self.themes.1)
				.render(bar_block[3], buf);
		}

		self.bar.render(bar_block[0], buf);
		get_paragraph(&self.percentage)
			.style(self.themes.1)
			.render(bar_block[1], buf);

		if !self.dpkg {
			get_paragraph(&self.current_total)
				.style(self.themes.0)
				.render(bar_block[2], buf);
		}
	}
}

pub(crate) struct TuiProgressRenderer<'a> {
	terminal: Term,
	config: &'a Config,
	ansi: Regex,
}

impl<'a> TuiProgressRenderer<'a> {
	pub(crate) fn new(config: &'a Config, terminal: Term) -> Result<Self> {
		Ok(Self {
			terminal,
			config,
			ansi: Regex::new(r"\x1b\[([\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e])")?,
		})
	}

	pub(crate) fn hide(&mut self) -> Result<()> {
		self.terminal.clear()?;
		self.terminal.show_cursor()?;
		Ok(())
	}

	pub(crate) fn unhide(&mut self) -> Result<()> {
		writeln!(stdout(), "\n\n\n")?;
		self.terminal.hide_cursor()?;
		Ok(())
	}

	pub(crate) fn clean_up(&mut self) -> Result<()> {
		self.terminal.clear()?;
		self.terminal.show_cursor()?;
		Ok(())
	}

	pub(crate) fn print(&mut self, state: &ProgressState, msg: &str) -> Result<()> {
		if state.hidden() {
			return Ok(());
		}

		// Strip ansi escape codes to get the correct size of the message
		let height = self.ansi.replace_all(msg, "").len() as f32
			/ self.terminal.backend().size()?.width as f32;

		// Check how many new lines as well
		let lines = (height.ceil() as u16).max(msg.lines().count() as u16);

		// Artifacts come into play if the viewport isn't cleared
		self.terminal.clear()?;
		self.terminal.insert_before(lines, |buf| {
			Paragraph::new(msg)
				.left_aligned()
				.wrap(Wrap::default())
				.style(tui_style::style(self.config, Theme::Regular))
				.render(buf.area, buf);
		})?;
		// Must redraw the terminal after printing
		self.render(state)
	}

	fn remaining_label(&self, state: &ProgressState) -> ProgressMessage {
		let mut msg = ProgressMessage::empty("Remaining: ");
		if let Some(eta) = state.eta() {
			msg.add(time_str(eta));
		}
		msg
	}

	pub(crate) fn render(&mut self, state: &ProgressState) -> Result<()> {
		if state.hidden() {
			return Ok(());
		}

		let progress = ProgressWidget {
			dpkg: state.is_dpkg(),
			percentage: format!("{:.1}%", state.ratio() * 100.0),
			current_total: state.current_total(),
			per_sec: format!("{}/s", state.unit_str(state.rate())),
			bar: LineGauge::default()
				.line_set(symbols::line::THICK)
				.ratio(state.ratio())
				.label(progress_line(&self.remaining_label(state), self.config))
				.filled_style(tui_style::style(self.config, Theme::ProgressFilled))
				.unfilled_style(tui_style::style(self.config, Theme::ProgressUnfilled)),
			spans: display_lines(state.display(), self.config),
			themes: (
				tui_style::style(self.config, Theme::Primary),
				tui_style::style(self.config, Theme::Secondary),
			),
		};

		self.terminal
			.draw(|f| progress.render(f.area(), f.buffer_mut()))?;

		Ok(())
	}
}

fn get_paragraph(text: &str) -> Paragraph<'_> { Paragraph::new(text).right_aligned() }

fn progress_line(msg: &ProgressMessage, config: &Config) -> Line<'static> {
	let mut line = Line::default();
	line.push_span(
		Span::from(msg.header().to_string()).style(tui_style::reset(config, msg.theme_value())),
	);

	for segment in msg.segments() {
		line.push_span(
			Span::from(segment.to_string()).style(tui_style::reset(config, Theme::Regular)),
		);
	}

	line
}

fn display_lines(display: &DisplayGroup, config: &Config) -> Vec<Line<'static>> {
	if display.messages().is_empty() {
		return vec![Line::from("Working...")];
	}

	display
		.messages()
		.iter()
		.map(|msg| progress_line(msg, config))
		.collect()
}
