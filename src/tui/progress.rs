use std::borrow::Cow;

use anyhow::Result;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::symbols::{self, border};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget, Wrap};
use rust_apt::util::time_str;

use crate::config::color::ansi_to_text;
use crate::config::{Config, Theme};
use crate::progress::{
	MAX_VISIBLE_MIRRORS, MirrorProgress, MirrorState, ProgressMessage, ProgressState, ProgressView,
};
use crate::t;
use crate::terminal::InlineTerminalGuard;

const WIDE_LAYOUT: u16 = 72;
const MAX_VIEW_WIDTH: u16 = 80;
const DIVIDER_BORDER: border::Set<'static> = border::Set {
	top_left: "├",
	top_right: "┤",
	bottom_left: "",
	bottom_right: "",
	vertical_left: "",
	vertical_right: "",
	horizontal_top: symbols::line::NORMAL.horizontal,
	horizontal_bottom: "",
};

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

		self.terminal.terminal_mut().draw(|frame| {
			let area = frame.area();
			render_progress_view(frame.buffer_mut(), area, self.config, state)
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

fn view_title(view: ProgressView) -> String {
	match view {
		ProgressView::Update => t!("progress-update-title"),
		ProgressView::Download { .. } => t!("progress-download-title"),
		ProgressView::Install => t!("progress-install-title"),
		ProgressView::MirrorScore => t!("progress-score-title"),
	}
}

pub(crate) fn render_progress_view(
	buf: &mut Buffer,
	mut area: Rect,
	config: &Config,
	state: &ProgressState,
) {
	area.width = area.width.min(MAX_VIEW_WIDTH);
	let title = view_title(state.view());
	let block = Block::bordered()
		.border_type(BorderType::Rounded)
		.padding(Padding::horizontal(1))
		.style(super::style::style(config, Theme::Primary))
		.title(
			Line::styled(
				format!("  {title}  "),
				super::style::style(config, Theme::Highlight),
			)
			.centered(),
		)
		.title_alignment(Alignment::Center);
	let inner = block.inner(area);
	block.render(area, buf);

	match state.view() {
		ProgressView::Download { mirrors } => {
			render_download(buf, config, area, inner, state, mirrors)
		},
		ProgressView::Update => render_update(buf, config, area, inner, state),
		ProgressView::Install => render_install(buf, config, area, inner, state),
		ProgressView::MirrorScore => render_mirror_score(buf, config, area, inner, state),
	}
}

fn render_download(
	buf: &mut Buffer,
	config: &Config,
	frame: Rect,
	area: Rect,
	state: &ProgressState,
	configured_mirrors: usize,
) {
	let visible = configured_mirrors.min(MAX_VISIBLE_MIRRORS) as u16;
	let overflow = u16::from(configured_mirrors > MAX_VISIBLE_MIRRORS);
	let mirror_height = u16::from(configured_mirrors > 0) + visible + overflow;
	let [mirrors, message, progress] = Layout::vertical([
		Constraint::Length(mirror_height),
		Constraint::Length(1),
		Constraint::Length(4),
	])
	.areas(area);

	render_mirrors(buf, config, mirrors, state, configured_mirrors);
	render_message(
		buf,
		config,
		message,
		state.message(),
		&t!("progress-last-completed"),
	);

	let (current, total) = state.items().unwrap_or_default();
	let left = [
		InfoRow::new(t!("progress-packages"), format!("{current}/{total}")),
		InfoRow::new(
			t!("progress-remaining"),
			state.eta().map_or_else(|| "—".to_string(), time_str),
		),
	];
	let right = [
		InfoRow::new(t!("progress-data"), state.current_total()),
		InfoRow::new(
			t!("progress-speed"),
			format!("{}/s", state.unit_str(state.rate())),
		),
	];
	render_progress_body(buf, config, frame, progress, state, &left, &right);
}

fn render_update(
	buf: &mut Buffer,
	config: &Config,
	frame: Rect,
	area: Rect,
	state: &ProgressState,
) {
	let left = [
		InfoRow::new(t!("progress-data"), state.current_total()),
		InfoRow::new(
			t!("progress-remaining"),
			state.eta().map_or_else(|| "—".to_string(), time_str),
		),
	];
	let right = [
		InfoRow::new(
			t!("progress-speed"),
			format!("{}/s", state.unit_str(state.rate())),
		),
		InfoRow::new(t!("progress-elapsed"), time_str(state.elapsed())),
	];
	render_standard(buf, config, frame, area, state, &left, &right);
}

fn render_install(
	buf: &mut Buffer,
	config: &Config,
	frame: Rect,
	area: Rect,
	state: &ProgressState,
) {
	let left = [InfoRow::new(
		t!("progress-package"),
		state.item().unwrap_or("—"),
	)];
	let right = [InfoRow::new(
		t!("progress-elapsed"),
		time_str(state.elapsed()),
	)];
	render_standard(buf, config, frame, area, state, &left, &right);
}

fn render_mirror_score(
	buf: &mut Buffer,
	config: &Config,
	frame: Rect,
	area: Rect,
	state: &ProgressState,
) {
	let left = [InfoRow::new(t!("progress-mirrors"), state.current_total())];
	let right = [InfoRow::new(
		t!("progress-elapsed"),
		time_str(state.elapsed()),
	)];
	render_standard(buf, config, frame, area, state, &left, &right);
}

fn render_standard(
	buf: &mut Buffer,
	config: &Config,
	frame: Rect,
	area: Rect,
	state: &ProgressState,
	left: &[InfoRow<'_>],
	right: &[InfoRow<'_>],
) {
	let body_height = 2 + left.len().max(right.len()) as u16;
	let [message, progress] =
		Layout::vertical([Constraint::Length(1), Constraint::Length(body_height)]).areas(area);
	render_message(
		buf,
		config,
		message,
		state.message(),
		&t!("progress-status"),
	);
	render_progress_body(buf, config, frame, progress, state, left, right);
}

fn render_message(
	buf: &mut Buffer,
	config: &Config,
	area: Rect,
	message: Option<&ProgressMessage>,
	default_label: &str,
) {
	if area.is_empty() {
		return;
	}

	let line = message.map_or_else(
		|| {
			Line::from(vec![
				Span::styled(
					format!(" {default_label}: "),
					super::style::reset(config, Theme::Primary),
				),
				Span::styled("—", super::style::reset(config, Theme::Regular)),
			])
		},
		|message| {
			let mut line = progress_line(message, config);
			line.spans.insert(0, Span::raw(" "));
			line
		},
	);
	Paragraph::new(line).render(area, buf);
}

fn render_mirrors(
	buf: &mut Buffer,
	config: &Config,
	area: Rect,
	state: &ProgressState,
	configured: usize,
) {
	if configured == 0 || area.is_empty() {
		return;
	}

	let visible = configured.min(MAX_VISIBLE_MIRRORS);
	let overflow = usize::from(configured > MAX_VISIBLE_MIRRORS);
	let rows = Layout::vertical(vec![Constraint::Length(1); 1 + visible + overflow]).split(area);
	render_mirror_header(buf, config, rows[0]);

	for (row, mirror) in rows[1..].iter().zip(state.mirrors().iter().take(visible)) {
		render_mirror_row(buf, config, *row, state, mirror);
	}

	if overflow > 0 {
		let hidden = configured - MAX_VISIBLE_MIRRORS;
		let active = state
			.mirrors()
			.iter()
			.map(MirrorProgress::active)
			.sum::<usize>();
		Paragraph::new(Line::styled(
			format!(
				" {}",
				t!(
					"progress-more-mirrors",
					"mirrors" => hidden,
					"connections" => active
				)
			),
			super::style::reset(config, Theme::Secondary),
		))
		.render(rows[rows.len() - 1], buf);
	}
}

fn mirror_columns(area: Rect) -> Vec<Rect> {
	if area.width >= WIDE_LAYOUT {
		Layout::horizontal([
			Constraint::Min(16),
			Constraint::Length(16),
			Constraint::Length(16),
			Constraint::Length(14),
		])
		.split(area)
		.to_vec()
	} else {
		Layout::horizontal([
			Constraint::Min(12),
			Constraint::Length(8),
			Constraint::Length(13),
		])
		.split(area)
		.to_vec()
	}
}

fn render_mirror_header(buf: &mut Buffer, config: &Config, area: Rect) {
	let columns = mirror_columns(area);
	let labels = if columns.len() == 4 {
		vec![
			t!("progress-mirrors"),
			t!("progress-connections"),
			t!("progress-average"),
			t!("progress-state"),
		]
	} else {
		vec![
			t!("progress-mirrors"),
			t!("progress-active"),
			t!("progress-state"),
		]
	};

	for (column, label) in columns.into_iter().zip(labels) {
		Paragraph::new(Line::styled(
			format!(" {label}"),
			super::style::reset(config, Theme::Primary),
		))
		.render(column, buf);
	}
}

fn render_mirror_row(
	buf: &mut Buffer,
	config: &Config,
	area: Rect,
	state: &ProgressState,
	mirror: &MirrorProgress,
) {
	let columns = mirror_columns(area);
	let regular = super::style::reset(config, Theme::Regular);
	Paragraph::new(Line::styled(format!(" {}", mirror.name()), regular)).render(columns[0], buf);

	let active = format!("{}/{}", mirror.active(), mirror.limit());
	Paragraph::new(Line::styled(format!(" {active}"), regular)).render(columns[1], buf);

	let status = match mirror.state() {
		MirrorState::Starting => t!("progress-starting"),
		MirrorState::Downloading => t!("progress-downloading"),
		MirrorState::Idle => t!("progress-idle"),
	};
	let status_theme = match mirror.state() {
		MirrorState::Starting => Theme::Notice,
		MirrorState::Downloading => Theme::Primary,
		MirrorState::Idle => Theme::Secondary,
	};

	if columns.len() == 4 {
		let rate = mirror.rate().map_or_else(
			|| "—".to_string(),
			|rate| format!("{}/s", state.unit_str(rate)),
		);
		Paragraph::new(Line::styled(format!(" {rate}"), regular)).render(columns[2], buf);
		Paragraph::new(Line::styled(
			format!(" {status}"),
			super::style::reset(config, status_theme),
		))
		.render(columns[3], buf);
	} else {
		Paragraph::new(Line::styled(
			format!(" {status}"),
			super::style::reset(config, status_theme),
		))
		.render(columns[2], buf);
	}
}

fn render_progress_body(
	buf: &mut Buffer,
	config: &Config,
	frame: Rect,
	area: Rect,
	state: &ProgressState,
	left: &[InfoRow<'_>],
	right: &[InfoRow<'_>],
) {
	if area.is_empty() {
		return;
	}

	let info_height = left.len().max(right.len()) as u16;
	let [divider, bar, info] = Layout::vertical([
		Constraint::Length(1),
		Constraint::Length(1),
		Constraint::Length(info_height),
	])
	.areas(area);
	let divider = Rect::new(frame.x, divider.y, frame.width, divider.height);
	render_progress_divider(buf, config, divider, state);
	render_progress_bar(buf, config, bar, state.ratio());
	render_info(buf, config, info, left, right);
}

fn render_progress_divider(buf: &mut Buffer, config: &Config, area: Rect, state: &ProgressState) {
	let percent = (state.ratio() * 100.0) as u64;
	Block::new()
		.borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
		.border_set(DIVIDER_BORDER)
		.border_style(super::style::style(config, Theme::Primary))
		.title(
			Line::styled(
				format!(" {} ", t!("progress-label")),
				super::style::reset(config, Theme::Highlight),
			)
			.centered(),
		)
		.title(
			Line::styled(
				format!(" {percent}% "),
				super::style::reset(config, Theme::Highlight),
			)
			.right_aligned(),
		)
		.render(area, buf);
}

fn render_progress_bar(buf: &mut Buffer, config: &Config, area: Rect, ratio: f64) {
	if area.is_empty() {
		return;
	}

	let filled = (f64::from(area.width) * ratio).round() as usize;
	let filled = filled.min(usize::from(area.width));
	let unfilled = usize::from(area.width) - filled;
	let line = Line::from(vec![
		Span::styled(
			symbols::line::THICK.horizontal.repeat(filled),
			super::style::style(config, Theme::ProgressFilled),
		),
		Span::styled(
			symbols::line::THICK.horizontal.repeat(unfilled),
			super::style::style(config, Theme::ProgressUnfilled),
		),
	]);
	Paragraph::new(line).render(area, buf);
}

fn render_info(
	buf: &mut Buffer,
	config: &Config,
	area: Rect,
	left: &[InfoRow<'_>],
	right: &[InfoRow<'_>],
) {
	let [left_area, right_area] =
		Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);
	render_info_column(buf, config, left_area, left);
	render_info_column(buf, config, right_area, right);
}

fn render_info_column(buf: &mut Buffer, config: &Config, area: Rect, rows: &[InfoRow<'_>]) {
	let label_width = rows
		.iter()
		.map(|row| Line::raw(row.label.as_ref()).width())
		.max()
		.unwrap_or_default();
	let slots = Layout::vertical(vec![Constraint::Length(1); rows.len()]).split(area);

	for (slot, row) in slots.iter().zip(rows) {
		let mut line = Line::from(" ");
		line.push_span(Span::styled(
			format!("{}:", row.label),
			super::style::reset(config, Theme::Primary),
		));
		let width = Line::raw(row.label.as_ref()).width();
		line.push_span(Span::raw(" ".repeat(label_width - width + 2)));
		line.push_span(Span::styled(
			row.value.as_ref(),
			super::style::reset(config, Theme::Regular),
		));
		Paragraph::new(line).render(*slot, buf);
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
