use std::borrow::Cow;

use anyhow::Result;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::symbols;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{LineGauge, Paragraph, Widget, Wrap};
use rust_apt::util::time_str;

use super::{borderless_area, frame_block};
use crate::config::color::ansi_to_text;
use crate::config::{Config, Theme};
use crate::progress::{DisplayGroup, ProgressMessage, ProgressPanel, ProgressState};
use crate::t;
use crate::terminal::InlineTerminalGuard;

struct InfoRow<'a> {
	label: Cow<'a, str>,
	value: Cow<'a, str>,
}

impl<'a> InfoRow<'a> {
	fn new(label: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
		Self {
			label: label.into(),
			value: value.into(),
		}
	}
}

pub(crate) struct TuiProgressRenderer<'a> {
	terminal: InlineTerminalGuard,
	config: &'a Config,
}

impl<'a> TuiProgressRenderer<'a> {
	pub(crate) fn new(config: &'a Config, lines: u16) -> Result<Self> {
		Ok(Self {
			terminal: InlineTerminalGuard::new(lines)?,
			config,
		})
	}

	pub(crate) fn hide(&mut self) -> Result<()> { self.terminal.hide() }

	pub(crate) fn suspend(&mut self) -> Result<()> { self.terminal.suspend() }

	pub(crate) fn resume(&mut self) -> Result<()> { self.terminal.resume() }

	pub(crate) fn print(&mut self, state: &ProgressState, msg: &str) -> Result<()> {
		if state.hidden() {
			return Ok(());
		}

		let terminal = self.terminal.terminal_mut();
		terminal.autoresize()?;
		let text = ansi_to_text(msg);
		let width = terminal.get_frame().area().width;
		let lines = message_height(&text, width);
		let paragraph = Paragraph::new(text)
			.left_aligned()
			.wrap(Wrap::default())
			.style(super::style::style(self.config, Theme::Regular));

		terminal.clear()?;
		terminal.insert_before(lines, move |buf| {
			paragraph.render(buf.area, buf);
		})?;
		self.render(state)
	}

	pub(crate) fn render(&mut self, state: &ProgressState) -> Result<()> {
		if state.hidden() {
			return Ok(());
		}

		let status_lines = display_lines(state.display(), self.config);
		let (left_info, right_info) = info_columns(state);

		self.terminal.terminal_mut().draw(|f| {
			render_progress_view(
				f,
				f.area(),
				self.config,
				state,
				&status_lines,
				&left_info,
				&right_info,
			)
		})?;

		Ok(())
	}
}

fn message_height(text: &Text, width: u16) -> u16 {
	let width = usize::from(width).max(1);
	text.lines
		.iter()
		.map(|line| line.width().div_ceil(width).max(1))
		.sum::<usize>()
		.min(usize::from(u16::MAX)) as u16
}

fn progress_line(msg: &ProgressMessage, config: &Config) -> Line<'static> {
	let mut line = Line::default();
	line.push_span(
		Span::from(msg.header().to_string()).style(super::style::reset(config, msg.theme_value())),
	);

	for segment in msg.segments() {
		line.push_span(
			Span::from(segment.to_string()).style(super::style::reset(config, Theme::Regular)),
		);
	}

	line
}

fn display_lines(display: &DisplayGroup, config: &Config) -> Vec<Line<'static>> {
	display
		.messages()
		.iter()
		.map(|msg| progress_line(msg, config))
		.collect()
}

fn info_columns(state: &ProgressState) -> (Vec<InfoRow<'_>>, Vec<InfoRow<'_>>) {
	let mut left = if state.is_dpkg() {
		vec![InfoRow::new(t!("progress-label"), state.current_total())]
	} else {
		vec![
			InfoRow::new(t!("progress-total"), state.current_total()),
			InfoRow::new(
				t!("progress-speed"),
				format!("{}/s", state.unit_str(state.rate())),
			),
		]
	};

	let mut right = if state.is_dpkg() {
		vec![InfoRow::new(
			t!("progress-elapsed"),
			time_str(state.elapsed()),
		)]
	} else {
		vec![
			InfoRow::new(t!("progress-elapsed"), time_str(state.elapsed())),
			match state.eta() {
				Some(eta) => InfoRow::new(t!("progress-remaining"), time_str(eta)),
				None => InfoRow::new(t!("progress-remaining"), "--"),
			},
		]
	};

	for (index, (label, value)) in state.info().iter().enumerate() {
		let row = InfoRow::new(label.as_str(), value.as_str());
		if index % 2 == 0 {
			left.push(row);
		} else {
			right.push(row);
		}
	}

	(left, right)
}

fn render_progress_view(
	f: &mut ratatui::Frame,
	area: Rect,
	config: &Config,
	state: &ProgressState,
	status_lines: &[Line<'static>],
	left_info: &[InfoRow<'_>],
	right_info: &[InfoRow<'_>],
) {
	let block = frame_block(config);
	let inner = block.inner(area);
	f.render_widget(block, area);

	let mirror_height = mirrors_height(state.panels());
	let status_height = status_lines.len() as u16;
	let progress_height = progress_height(left_info, right_info);

	let mut constraints = Vec::with_capacity(4);
	if mirror_height > 0 {
		constraints.push(Constraint::Length(mirror_height));
	}
	if status_height > 0 {
		constraints.push(Constraint::Length(status_height));
	}
	constraints.push(Constraint::Length(progress_height));
	constraints.push(Constraint::Min(0));

	let slots = Layout::vertical(constraints).split(inner);
	let mut index = 0;

	if mirror_height > 0 {
		render_mirrors(f, config, slots[index], state.panels());
		index += 1;
	}

	if status_height > 0 {
		render_status(f.buffer_mut(), slots[index], status_lines);
		index += 1;
	}

	render_progress_widget(
		f.buffer_mut(),
		config,
		slots[index],
		state,
		left_info,
		right_info,
	);
}

fn mirrors_height(panels: &[ProgressPanel]) -> u16 {
	if panels.is_empty() {
		0
	} else {
		1 + panels.iter().map(ProgressPanel::height).sum::<u16>()
	}
}

fn progress_height(left_info: &[InfoRow<'_>], right_info: &[InfoRow<'_>]) -> u16 {
	(1 + left_info.len().max(right_info.len()) as u16).max(4)
}

fn render_mirrors(f: &mut ratatui::Frame, config: &Config, area: Rect, panels: &[ProgressPanel]) {
	if panels.is_empty() || area.width == 0 || area.height == 0 {
		return;
	}

	let inner = borderless_area(f, area, &t!("mirrors"));
	let heights = panels
		.iter()
		.map(|panel| Constraint::Length(panel.height()))
		.collect::<Vec<_>>();
	let slots = Layout::vertical(heights).split(inner);

	for (panel, slot) in panels.iter().zip(slots.iter()) {
		render_panel(f, config, *slot, panel);
	}
}

fn render_panel(f: &mut ratatui::Frame, config: &Config, area: Rect, panel: &ProgressPanel) {
	if area.width == 0 || area.height == 0 {
		return;
	}

	let inner = borderless_area(f, area, panel.title());

	if panel.items().is_empty() || inner.width == 0 || inner.height == 0 {
		return;
	}

	let widths = vec![Constraint::Length(1); panel.items().len()];
	let slots = Layout::vertical(widths).split(inner);
	let key_width = panel.items().len().to_string().len() + 1;

	for (slot, (index, item)) in slots.iter().zip(panel.items().iter().enumerate()) {
		let number = (index + 1).to_string();
		let mut line = Line::default();
		line.push_span(
			Span::from(number.clone()).style(super::style::reset(config, Theme::Primary)),
		);
		line.push_span(Span::raw(" ".repeat(key_width - number.len())));
		line.push_span(Span::from(item.clone()).style(super::style::reset(config, Theme::Regular)));
		Paragraph::new(line)
			.wrap(Wrap { trim: false })
			.render(*slot, f.buffer_mut());
	}
}

fn render_status(buf: &mut Buffer, area: Rect, lines: &[Line<'static>]) {
	if lines.is_empty() || area.width == 0 || area.height == 0 {
		return;
	}

	let slots = Layout::vertical(vec![Constraint::Length(1); lines.len()]).split(area);
	for (slot, line) in slots.iter().zip(lines.iter()) {
		Paragraph::new(line.clone())
			.wrap(Wrap { trim: false })
			.render(*slot, buf);
	}
}

fn render_progress_widget(
	buf: &mut Buffer,
	config: &Config,
	area: Rect,
	state: &ProgressState,
	left_info: &[InfoRow<'_>],
	right_info: &[InfoRow<'_>],
) {
	if area.width == 0 || area.height == 0 {
		return;
	}

	let [bar_area, info_area] =
		Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
	let [bar_slot, _, _] = split_columns(bar_area);

	let bar = LineGauge::default()
		.filled_symbol(symbols::line::THICK.horizontal)
		.unfilled_symbol(symbols::line::THICK.horizontal)
		.ratio(state.ratio())
		.label(progress_line(
			&ProgressMessage::empty(format!("{}:", t!("progress-label"))),
			config,
		))
		.filled_style(super::style::style(config, Theme::ProgressFilled))
		.unfilled_style(super::style::style(config, Theme::ProgressUnfilled));
	bar.render(bar_slot, buf);

	let [left_area, right_area, _] = split_columns(info_area);
	render_info_column(buf, config, left_area, left_info);
	render_info_column(buf, config, right_area, right_info);
}

fn split_columns(area: Rect) -> [Rect; 3] {
	Layout::horizontal([Constraint::Max(32), Constraint::Max(32), Constraint::Min(0)]).areas(area)
}

fn render_info_column(buf: &mut Buffer, config: &Config, area: Rect, rows: &[InfoRow<'_>]) {
	if rows.is_empty() || area.width == 0 || area.height == 0 {
		return;
	}

	let labels = rows
		.iter()
		.map(|row| format!("  {}:", row.label))
		.collect::<Vec<_>>();
	let label_width = labels.iter().map(String::len).max().unwrap_or_default() + 1;
	let slots = Layout::vertical(vec![Constraint::Length(1); rows.len()]).split(area);

	for ((slot, label), row) in slots.iter().zip(labels.iter()).zip(rows.iter()) {
		let mut line = Line::default();
		line.push_span(
			Span::from(label.clone()).style(super::style::reset(config, Theme::Primary)),
		);
		line.push_span(Span::raw(" ".repeat(label_width - label.len())));
		line.push_span(
			Span::from(row.value.as_ref()).style(super::style::reset(config, Theme::Regular)),
		);
		Paragraph::new(line)
			.wrap(Wrap { trim: false })
			.render(*slot, buf);
	}
}

#[cfg(test)]
mod tests {
	use super::message_height;
	use crate::config::color::ansi_to_text;

	#[test]
	fn message_height_uses_display_width() {
		let isolated = "\u{2068}1234567890\u{2069}";
		assert_eq!(message_height(&ansi_to_text(isolated), 10), 1);
	}
}
