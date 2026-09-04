//! Shared progress runtime.
//!
//! This module owns the progress state model used by call sites, the plain
//! progress renderer, and the selection of the ratatui progress backend.

use std::env;
use std::io::{IsTerminal, Write, stderr};
use std::time::Instant;

use anyhow::Result;
use rust_apt::util::time_str;

use crate::config::{Config, Theme};
use crate::t;
use crate::terminal::use_enhanced_ui;
use crate::tui::progress::TuiProgressRenderer;
use crate::util::{NumSys, UnitStr};

pub(crate) const MAX_VISIBLE_MIRRORS: usize = 3;

#[derive(Clone)]
pub(crate) struct ProgressMessage {
	header: String,
	theme: Theme,
	msg: Vec<String>,
}

impl ProgressMessage {
	pub fn new<T: ToString>(header: T, msg: Vec<String>) -> Self {
		Self {
			header: header.to_string(),
			theme: Theme::Primary,
			msg,
		}
	}

	pub fn theme(mut self, theme: Theme) -> Self {
		self.theme = theme;
		self
	}

	pub fn regular(self) -> Self { self.theme(Theme::Regular) }

	pub fn header(&self) -> &str { &self.header }

	pub fn theme_value(&self) -> Theme { self.theme }

	pub fn segments(&self) -> &[String] { &self.msg }

	fn plain_line(&self) -> String {
		let mut line = String::with_capacity(
			self.header.len() + self.msg.iter().map(String::len).sum::<usize>(),
		);
		line.push_str(&self.header);
		for msg in &self.msg {
			line.push_str(msg);
		}
		line
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProgressView {
	Update,
	Download { mirrors: usize },
	Install,
	MirrorScore,
}

impl ProgressView {
	fn viewport_lines(self) -> u16 {
		match self {
			Self::Update => 7,
			Self::Install | Self::MirrorScore => 6,
			Self::Download { mirrors } => {
				let visible = mirrors.min(MAX_VISIBLE_MIRRORS);
				let overflow = usize::from(mirrors > MAX_VISIBLE_MIRRORS);
				let table = usize::from(mirrors > 0) + visible + overflow;
				(7 + table) as u16
			},
		}
	}

	pub(crate) fn is_transfer(self) -> bool { matches!(self, Self::Update | Self::Download { .. }) }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MirrorState {
	Starting,
	Downloading,
	Idle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MirrorProgress {
	name: String,
	active: usize,
	limit: usize,
	rate: Option<u64>,
	state: MirrorState,
}

impl MirrorProgress {
	pub(crate) fn new(
		name: String,
		active: usize,
		limit: usize,
		rate: Option<u64>,
		seen: bool,
	) -> Self {
		let state = if active > 0 && rate.is_some() {
			MirrorState::Downloading
		} else if active > 0 || !seen {
			MirrorState::Starting
		} else {
			MirrorState::Idle
		};

		Self {
			name,
			active,
			limit,
			rate,
			state,
		}
	}

	pub(crate) fn name(&self) -> &str { &self.name }

	pub(crate) fn active(&self) -> usize { self.active }

	pub(crate) fn limit(&self) -> usize { self.limit }

	pub(crate) fn rate(&self) -> Option<u64> { self.rate }

	pub(crate) fn state(&self) -> MirrorState { self.state }
}

pub(crate) struct ProgressState {
	length: u64,
	position: u64,
	transferred: u64,
	started: Instant,
	view: ProgressView,
	message: Option<ProgressMessage>,
	item: Option<String>,
	items: Option<(usize, usize)>,
	mirrors: Vec<MirrorProgress>,
	hidden: bool,
	unit: UnitStr,
}

impl ProgressState {
	fn new(view: ProgressView) -> Self {
		Self {
			length: 0,
			position: 0,
			transferred: 0,
			started: Instant::now(),
			view,
			message: None,
			item: None,
			items: None,
			mirrors: vec![],
			hidden: false,
			unit: UnitStr::new(1, NumSys::Binary),
		}
	}

	fn set_length(&mut self, len: u64) { self.length = len }

	fn inc_length(&mut self, delta: u64) { self.length = self.length.saturating_add(delta) }

	fn inc(&mut self, delta: u64) {
		self.position = self.position.saturating_add(delta);
		self.transferred = self.transferred.saturating_add(delta);
	}

	fn inc_cached(&mut self, delta: u64) { self.position = self.position.saturating_add(delta); }

	fn dec(&mut self, delta: u64) {
		self.position = self.position.saturating_sub(delta);
		self.transferred = self.transferred.saturating_sub(delta);
	}

	fn set_position(&mut self, pos: u64) {
		self.position = pos;
		self.transferred = pos;
	}

	fn finish(&mut self) { self.position = self.length }

	pub(crate) fn view(&self) -> ProgressView { self.view }

	pub(crate) fn message(&self) -> Option<&ProgressMessage> { self.message.as_ref() }

	fn set_message(&mut self, message: ProgressMessage) { self.message = Some(message) }

	pub(crate) fn item(&self) -> Option<&str> { self.item.as_deref() }

	fn set_item(&mut self, item: String) { self.item = Some(item) }

	pub(crate) fn items(&self) -> Option<(usize, usize)> { self.items }

	fn set_items(&mut self, current: usize, total: usize) { self.items = Some((current, total)) }

	pub(crate) fn mirrors(&self) -> &[MirrorProgress] { &self.mirrors }

	fn set_mirrors(&mut self, mirrors: Vec<MirrorProgress>) { self.mirrors = mirrors }

	pub(crate) fn hidden(&self) -> bool { self.hidden }

	fn set_hidden(&mut self, hidden: bool) { self.hidden = hidden }

	pub(crate) fn unit_str(&self, size: u64) -> String { self.unit.str(size) }

	pub(crate) fn current_total(&self) -> String {
		if self.view.is_transfer() {
			format!(
				"{}/{}",
				self.unit.str(self.position),
				self.unit.str(self.length),
			)
		} else {
			format!("{}/{}", self.position, self.length)
		}
	}

	pub(crate) fn elapsed(&self) -> u64 { self.started.elapsed().as_secs_f64().ceil() as u64 }

	pub(crate) fn rate(&self) -> u64 {
		let elapsed = self.started.elapsed().as_secs_f64();
		if elapsed <= 0.0 {
			return self.transferred;
		}
		(self.transferred as f64 / elapsed).ceil() as u64
	}

	pub(crate) fn eta(&self) -> Option<u64> {
		if self.position == 0 || self.position >= self.length {
			return None;
		}

		let rate = self.rate();
		if rate == 0 {
			return None;
		}

		Some(((self.length - self.position) as f64 / rate as f64).ceil() as u64)
	}

	pub(crate) fn ratio(&self) -> f64 {
		if self.length == 0 {
			return 0.0;
		}
		(self.position as f64 / self.length as f64).min(1.0)
	}

	fn finished_string(&self) -> String {
		if self.length > 1 {
			let rate = self.rate();
			t!(
				"progress-fetched",
				"size" => self.unit.str(self.length),
				"time" => time_str(self.elapsed()),
				"rate" => self.unit.str(rate),
			)
		} else {
			t!("progress-nothing")
		}
	}
}

pub(crate) struct PlainProgress {
	interactive: bool,
	line_rendered: bool,
}

impl PlainProgress {
	fn new() -> Self {
		let dumb_term = env::var("TERM")
			.ok()
			.is_some_and(|term| term.eq_ignore_ascii_case("dumb"));

		Self {
			interactive: stderr().is_terminal() && !dumb_term,
			line_rendered: false,
		}
	}

	fn bar(&self, state: &ProgressState) -> String {
		const WIDTH: usize = 40;

		let filled = (state.ratio() * WIDTH as f64).round() as usize;
		let filled = filled.min(WIDTH);
		format!("[{}{}]", "=".repeat(filled), " ".repeat(WIDTH - filled))
	}

	fn line(&self, state: &ProgressState) -> String {
		let mut line = format!(
			"{} {:>3}% ",
			self.bar(state),
			(state.ratio() * 100.0) as u64
		);
		let mut message = state
			.message()
			.map_or_else(|| t!("progress-working"), ProgressMessage::plain_line);

		if state.view().is_transfer() {
			let rate = format!("{}/s", state.unit_str(state.rate()));
			if !message.is_empty() {
				message.push(' ');
			}
			message.push_str(&state.current_total());
			message.push(' ');
			message.push_str(&rate);
		} else if state.view() == ProgressView::MirrorScore {
			message.push_str(" | ");
			message.push_str(&t!("progress-mirrors"));
			message.push_str(": ");
			message.push_str(&state.current_total());
		}

		if let Some(item) = state.item() {
			if !message.is_empty() {
				message.push_str(" | ");
			}
			message.push_str(&t!("progress-package"));
			message.push_str(": ");
			message.push_str(item);
		}

		if let Some((current, total)) = state.items() {
			if !message.is_empty() {
				message.push_str(" | ");
			}
			message.push_str(&t!("progress-packages"));
			message.push_str(": ");
			message.push_str(&format!("{current}/{total}"));
		}

		line.push_str(&message);
		line
	}

	fn clear_line(&mut self) -> Result<()> {
		if self.interactive && self.line_rendered {
			eprint!("\r\x1b[2K");
			stderr().flush()?;
			self.line_rendered = false;
		}
		Ok(())
	}

	fn print(&mut self, state: &ProgressState, msg: &str) {
		if state.hidden() {
			return;
		}

		let _ = self.clear_line();
		eprintln!("{msg}");
		if self.interactive {
			self.line_rendered = false;
		}
	}

	fn render(&mut self, state: &ProgressState) -> Result<()> {
		if !self.interactive || state.hidden() {
			return Ok(());
		}

		eprint!("\r\x1b[2K{}", self.line(state));
		stderr().flush()?;
		self.line_rendered = true;
		Ok(())
	}
}

enum ProgressKind<'a> {
	Tui(TuiProgressRenderer<'a>),
	Plain(PlainProgress),
}

pub(crate) struct Progress<'a> {
	state: ProgressState,
	kind: ProgressKind<'a>,
}

impl<'a> Progress<'a> {
	fn new(config: &'a Config, view: ProgressView) -> Result<Self> {
		let kind = if use_enhanced_ui(config) {
			ProgressKind::Tui(TuiProgressRenderer::new(config, view.viewport_lines())?)
		} else {
			ProgressKind::Plain(PlainProgress::new())
		};

		Ok(Self {
			state: ProgressState::new(view),
			kind,
		})
	}

	pub fn update(config: &'a Config) -> Result<Self> { Self::new(config, ProgressView::Update) }

	pub fn download(config: &'a Config, mirrors: usize) -> Result<Self> {
		Self::new(config, ProgressView::Download { mirrors })
	}

	pub fn install(config: &'a Config) -> Result<Self> { Self::new(config, ProgressView::Install) }

	pub fn mirror_score(config: &'a Config) -> Result<Self> {
		Self::new(config, ProgressView::MirrorScore)
	}

	pub fn set_length(&mut self, len: u64) { self.state.set_length(len) }

	pub fn inc_length(&mut self, delta: u64) { self.state.inc_length(delta) }

	pub fn inc(&mut self, delta: u64) { self.state.inc(delta) }

	pub fn inc_cached(&mut self, delta: u64) { self.state.inc_cached(delta) }

	pub fn dec(&mut self, delta: u64) { self.state.dec(delta) }

	pub fn set_position(&mut self, pos: u64) { self.state.set_position(pos) }

	pub fn finish(&mut self) { self.state.finish() }

	pub fn unit_str(&self, size: u64) -> String { self.state.unit_str(size) }

	pub fn set_message(&mut self, message: ProgressMessage) { self.state.set_message(message) }

	pub fn set_item(&mut self, item: String) { self.state.set_item(item) }

	pub fn set_items(&mut self, current: usize, total: usize) {
		self.state.set_items(current, total)
	}

	pub fn set_mirrors(&mut self, mirrors: Vec<MirrorProgress>) { self.state.set_mirrors(mirrors) }

	pub fn hidden(&self) -> bool { self.state.hidden() }

	pub fn hide(&mut self) -> Result<()> {
		if self.state.hidden() {
			return Ok(());
		}

		match &mut self.kind {
			ProgressKind::Tui(renderer) => renderer.hide()?,
			ProgressKind::Plain(inner) => inner.clear_line()?,
		}

		self.state.set_hidden(true);
		Ok(())
	}

	pub fn unhide(&mut self) -> Result<()> {
		if !self.state.hidden() {
			return Ok(());
		}

		if let ProgressKind::Tui(renderer) = &mut self.kind {
			renderer.resume()?;
		}

		self.state.set_hidden(false);
		Ok(())
	}

	pub fn suspend(&mut self) -> Result<()> {
		if self.state.hidden() {
			return Ok(());
		}

		match &mut self.kind {
			ProgressKind::Tui(renderer) => renderer.suspend()?,
			ProgressKind::Plain(inner) => inner.clear_line()?,
		}

		self.state.set_hidden(true);
		Ok(())
	}

	pub fn resume(&mut self) -> Result<()> {
		if !self.state.hidden() {
			return Ok(());
		}

		if let ProgressKind::Tui(renderer) = &mut self.kind {
			renderer.resume()?;
		}

		self.state.set_hidden(false);
		self.render()
	}

	pub fn print(&mut self, msg: &str) -> Result<()> {
		let state = &self.state;
		match &mut self.kind {
			ProgressKind::Tui(renderer) => renderer.print(state, msg),
			ProgressKind::Plain(inner) => {
				inner.print(state, msg);
				Ok(())
			},
		}
	}

	pub fn render(&mut self) -> Result<()> {
		let state = &self.state;
		match &mut self.kind {
			ProgressKind::Tui(renderer) => renderer.render(state),
			ProgressKind::Plain(inner) => inner.render(state),
		}
	}

	pub fn clean_up(&mut self) -> Result<()> {
		match &mut self.kind {
			ProgressKind::Tui(renderer) => renderer.suspend(),
			ProgressKind::Plain(inner) => inner.clear_line(),
		}
	}

	pub fn finished_string(&self) -> String { self.state.finished_string() }
}

#[cfg(test)]
mod tests {
	use ratatui::buffer::Buffer;
	use ratatui::layout::Rect;

	use super::{MirrorProgress, MirrorState, ProgressMessage, ProgressState, ProgressView};
	use crate::config::Config;
	use crate::tui::progress::render_progress_view;

	fn lines(buf: &Buffer) -> Vec<String> {
		(0..buf.area.height)
			.map(|y| {
				(0..buf.area.width)
					.map(|x| buf.cell((x, y)).unwrap().symbol())
					.collect::<String>()
					.trim_end()
					.to_string()
			})
			.collect()
	}

	#[test]
	fn download_view_has_no_unused_rows() {
		let view = ProgressView::Download { mirrors: 4 };
		let mut state = ProgressState::new(view);
		state.set_length(100);
		state.set_position(73);
		state.set_items(43, 80);
		state.set_message(ProgressMessage::new(
			"Last completed: ",
			vec!["libssl3t64.deb".into()],
		));
		state.set_mirrors(vec![
			MirrorProgress::new("deb.debian.org".into(), 3, 3, Some(12), true),
			MirrorProgress::new("deb.volian.org".into(), 2, 3, Some(7), true),
			MirrorProgress::new("mirror.example".into(), 0, 3, None, false),
			MirrorProgress::new("fallback.example".into(), 0, 3, None, false),
		]);

		let area = Rect::new(0, 0, 100, view.viewport_lines());
		let mut buf = Buffer::empty(area);
		render_progress_view(&mut buf, area, &Config::default(), &state);
		let lines = lines(&buf);

		assert_eq!(lines.len(), 12);
		assert!(lines[0].contains("Downloading Packages"));
		assert!(lines[1].contains("Connections"));
		assert!(lines[2].contains("deb.debian.org"));
		assert!(lines[3].contains("deb.volian.org"));
		assert!(lines[4].contains("mirror.example"));
		assert!(lines[5].contains("+1 more mirrors"));
		assert!(lines[6].contains("libssl3t64.deb"));
		assert!(lines[7].contains("Progress"));
		assert!(lines[7].contains("73%"));
		assert!(lines[7].starts_with('├'));
		assert!(lines[7].ends_with('┤'));
		assert!(lines[9].contains("Packages:"));
		assert!(lines[9].contains("Data:"));
		assert!(lines[10].contains("Remaining:"));
		assert!(lines[10].contains("Speed:"));
		assert!(lines[11].starts_with('╰'));
		assert_eq!(lines[1].find("Connections"), lines[2].find("3/3"),);
		assert_eq!(lines[1].find("Average"), lines[2].find("12 B/s"));
		assert_eq!(lines[1].find("State"), lines[2].find("downloading"),);
	}

	#[test]
	fn download_height_follows_known_mirror_count() {
		assert_eq!(ProgressView::Download { mirrors: 0 }.viewport_lines(), 7);
		assert_eq!(ProgressView::Download { mirrors: 1 }.viewport_lines(), 9);
		assert_eq!(ProgressView::Download { mirrors: 3 }.viewport_lines(), 11);
		assert_eq!(ProgressView::Download { mirrors: 8 }.viewport_lines(), 12);
	}

	#[test]
	fn mirror_state_comes_from_real_activity() {
		assert_eq!(
			MirrorProgress::new("new".into(), 0, 3, None, false).state(),
			MirrorState::Starting
		);
		assert_eq!(
			MirrorProgress::new("active".into(), 1, 3, Some(5), true).state(),
			MirrorState::Downloading
		);
		assert_eq!(
			MirrorProgress::new("done".into(), 0, 3, Some(5), true).state(),
			MirrorState::Idle
		);
	}

	#[test]
	fn cached_bytes_advance_progress_without_inflating_speed() {
		let mut state = ProgressState::new(ProgressView::Download { mirrors: 1 });
		state.inc_cached(1024);

		assert_eq!(state.position, 1024);
		assert_eq!(state.transferred, 0);
		assert_eq!(state.rate(), 0);

		state.inc(512);
		assert_eq!(state.position, 1536);
		assert_eq!(state.transferred, 512);
	}

	#[test]
	fn narrow_download_view_keeps_the_mirror_state_legible() {
		let view = ProgressView::Download { mirrors: 1 };
		let mut state = ProgressState::new(view);
		state.set_mirrors(vec![MirrorProgress::new(
			"deb.debian.org".into(),
			2,
			3,
			Some(1024),
			true,
		)]);

		let area = Rect::new(0, 0, 60, view.viewport_lines());
		let mut buf = Buffer::empty(area);
		render_progress_view(&mut buf, area, &Config::default(), &state);
		let lines = lines(&buf);

		assert!(lines[1].contains("Active"));
		assert!(lines[1].contains("State"));
		assert!(lines[2].contains("deb.debian.org"));
		assert!(lines[2].contains("2/3"));
		assert!(lines[2].contains("downloading"));
	}

	#[test]
	fn wide_terminal_keeps_the_progress_view_left_aligned_and_eighty_columns_wide() {
		let view = ProgressView::Download { mirrors: 1 };
		let state = ProgressState::new(view);
		let area = Rect::new(0, 0, 140, view.viewport_lines());
		let mut buf = Buffer::empty(area);
		render_progress_view(&mut buf, area, &Config::default(), &state);

		assert_eq!(buf.cell((0, 0)).unwrap().symbol(), "╭");
		assert_eq!(buf.cell((79, 0)).unwrap().symbol(), "╮");
		assert_eq!(buf.cell((80, 0)).unwrap().symbol(), " ");
	}

	#[test]
	fn install_view_shows_current_package_and_status() {
		let view = ProgressView::Install;
		let mut state = ProgressState::new(view);
		state.set_length(100);
		state.set_position(40);
		state.set_item("hello".into());
		state.set_message(ProgressMessage::new(
			"Status: ",
			vec!["Configuring hello".into()],
		));

		let area = Rect::new(0, 0, 80, view.viewport_lines());
		let mut buf = Buffer::empty(area);
		render_progress_view(&mut buf, area, &Config::default(), &state);
		let lines = lines(&buf);

		assert_eq!(lines.len(), 6);
		assert!(lines[1].contains("Configuring hello"));
		assert!(lines[2].contains("40%"));
		assert!(lines[4].contains("Package:  hello"));
		assert!(lines[4].contains("Elapsed:"));
		assert!(lines[5].starts_with('╰'));
	}
}
