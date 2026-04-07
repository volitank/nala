//! Shared progress runtime.
//!
//! This module owns the progress state model used by call sites, the plain
//! progress renderer, and the selection of the ratatui progress backend.

use std::env;
use std::io::{stderr, stdout, IsTerminal, Write};
use std::time::Instant;

use anyhow::Result;
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};
use rust_apt::util::time_str;

use crate::config::{Config, Theme};
use crate::terminal::{use_tui, RawModeGuard, Term};
use crate::tui::progress::TuiProgressRenderer;
use crate::util::{NumSys, UnitStr};

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

	pub fn empty<T: ToString>(header: T) -> Self { Self::new(header, vec![]) }

	pub fn theme(mut self, theme: Theme) -> Self {
		self.theme = theme;
		self
	}

	pub fn regular(self) -> Self { self.theme(Theme::Regular) }

	pub fn add(&mut self, value: String) { self.msg.push(value) }

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

#[derive(Clone)]
pub(crate) struct DisplayGroup(Vec<ProgressMessage>);

impl DisplayGroup {
	pub fn new() -> Self { Self(vec![]) }

	pub fn clear(&mut self) -> &mut Self {
		self.0.clear();
		self
	}

	pub fn push(&mut self, value: ProgressMessage) -> &mut Self {
		self.0.push(value);
		self
	}

	pub fn push_str<T: ToString>(&mut self, header: T, value: String) -> &mut Self {
		self.push(ProgressMessage::new(header.to_string(), vec![value]))
	}

	pub(crate) fn messages(&self) -> &[ProgressMessage] { &self.0 }

	fn plain_lines(&self) -> Vec<String> {
		if self.0.is_empty() {
			vec!["Working...".to_string()]
		} else {
			self.0.iter().map(ProgressMessage::plain_line).collect()
		}
	}
}

pub(crate) struct ProgressState {
	length: u64,
	position: u64,
	started: Instant,
	display: DisplayGroup,
	hidden: bool,
	unit: UnitStr,
	dpkg: bool,
}

impl ProgressState {
	fn new(dpkg: bool) -> Self {
		Self {
			length: 0,
			position: 0,
			started: Instant::now(),
			display: DisplayGroup::new(),
			hidden: false,
			unit: UnitStr::new(1, NumSys::Binary),
			dpkg,
		}
	}

	fn set_length(&mut self, len: u64) { self.length = len }

	fn inc_length(&mut self, delta: u64) { self.length = self.length.saturating_add(delta) }

	fn inc(&mut self, delta: u64) { self.position = self.position.saturating_add(delta) }

	fn set_position(&mut self, pos: u64) { self.position = pos }

	fn finish(&mut self) { self.position = self.length }

	pub(crate) fn length(&self) -> u64 { self.length }

	pub(crate) fn is_dpkg(&self) -> bool { self.dpkg }

	pub(crate) fn display(&self) -> &DisplayGroup { &self.display }

	fn display_mut(&mut self) -> &mut DisplayGroup { &mut self.display }

	pub(crate) fn hidden(&self) -> bool { self.hidden }

	fn set_hidden(&mut self, hidden: bool) { self.hidden = hidden }

	pub(crate) fn unit_str(&self, size: u64) -> String { self.unit.str(size) }

	pub(crate) fn current_total(&self) -> String {
		if self.dpkg {
			format!("{}/{}", self.position, self.length)
		} else {
			format!(
				"{}/{}",
				self.unit.str(self.position),
				self.unit.str(self.length),
			)
		}
	}

	pub(crate) fn elapsed(&self) -> u64 { self.started.elapsed().as_secs_f64().ceil() as u64 }

	pub(crate) fn rate(&self) -> u64 {
		let elapsed = self.started.elapsed().as_secs_f64();
		if elapsed <= 0.0 {
			return self.position;
		}
		(self.position as f64 / elapsed).ceil() as u64
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
			format!(
				"Fetched {} in {} ({}/s)",
				self.unit.str(self.length),
				time_str(self.elapsed()),
				self.unit.str(rate),
			)
		} else {
			"Nothing to fetch".to_string()
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
		let mut message = state.display.plain_lines().join(" | ");

		if !state.is_dpkg() {
			let rate = format!("{}/s", state.unit_str(state.rate()));
			if !message.is_empty() {
				message.push(' ');
			}
			message.push_str(&state.current_total());
			message.push(' ');
			message.push_str(&rate);
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

	fn hide(&mut self) -> Result<()> {
		self.clear_line()?;
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

	fn clean_up(&mut self) -> Result<()> { self.clear_line() }
}

fn progress_terminal(dpkg: bool) -> Result<Term> {
	Ok(Terminal::with_options(
		CrosstermBackend::new(stdout()),
		TerminalOptions {
			viewport: Viewport::Inline(if dpkg { 3 } else { 5 }),
		},
	)?)
}

enum ProgressKind<'a> {
	Tui {
		renderer: TuiProgressRenderer<'a>,
		raw: RawModeGuard,
	},
	Plain(PlainProgress),
}

pub(crate) struct Progress<'a> {
	state: ProgressState,
	kind: ProgressKind<'a>,
}

impl<'a> Progress<'a> {
	pub fn new(config: &'a Config, dpkg: bool) -> Result<Self> {
		let kind = if use_tui(config) {
			let raw = RawModeGuard::new()?;
			let terminal = progress_terminal(dpkg)?;
			let renderer = TuiProgressRenderer::new(config, terminal)?;
			ProgressKind::Tui { renderer, raw }
		} else {
			ProgressKind::Plain(PlainProgress::new())
		};

		Ok(Self {
			state: ProgressState::new(dpkg),
			kind,
		})
	}

	pub fn set_length(&mut self, len: u64) { self.state.set_length(len) }

	pub fn inc_length(&mut self, delta: u64) { self.state.inc_length(delta) }

	pub fn inc(&mut self, delta: u64) { self.state.inc(delta) }

	pub fn set_position(&mut self, pos: u64) { self.state.set_position(pos) }

	pub fn finish(&mut self) { self.state.finish() }

	pub fn length(&self) -> u64 { self.state.length() }

	pub fn unit_str(&self, size: u64) -> String { self.state.unit_str(size) }

	pub fn display_mut(&mut self) -> &mut DisplayGroup { self.state.display_mut() }

	pub fn hidden(&self) -> bool { self.state.hidden() }

	pub fn hide(&mut self) -> Result<()> {
		if self.state.hidden() {
			return Ok(());
		}

		match &mut self.kind {
			ProgressKind::Tui { renderer, .. } => renderer.hide()?,
			ProgressKind::Plain(inner) => inner.hide()?,
		}

		self.state.set_hidden(true);
		Ok(())
	}

	pub fn unhide(&mut self) -> Result<()> {
		if !self.state.hidden() {
			return Ok(());
		}

		if let ProgressKind::Tui { renderer, .. } = &mut self.kind {
			renderer.unhide()?;
		}

		self.state.set_hidden(false);
		Ok(())
	}

	pub fn print(&mut self, msg: &str) -> Result<()> {
		let state = &self.state;
		match &mut self.kind {
			ProgressKind::Tui { renderer, .. } => renderer.print(state, msg),
			ProgressKind::Plain(inner) => {
				inner.print(state, msg);
				Ok(())
			},
		}
	}

	pub fn render(&mut self) -> Result<()> {
		let state = &self.state;
		match &mut self.kind {
			ProgressKind::Tui { renderer, .. } => renderer.render(state),
			ProgressKind::Plain(inner) => inner.render(state),
		}
	}

	pub fn clean_up(&mut self) -> Result<()> {
		match &mut self.kind {
			ProgressKind::Tui { renderer, raw } => {
				let renderer_result = renderer.clean_up();
				let raw_result = raw.disable();
				renderer_result?;
				raw_result
			},
			ProgressKind::Plain(inner) => inner.clean_up(),
		}
	}

	pub fn finished_string(&self) -> String { self.state.finished_string() }
}
