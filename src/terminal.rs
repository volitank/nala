//! Terminal runtime helpers.
//!
//! This module owns terminal/session mechanics such as TUI mode policy,
//! raw mode, alternate screen, mouse capture, and the shared terminal type.

use std::env;
use std::io::{IsTerminal, Write, stdin, stdout};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
	self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
};
use crossterm::terminal::{
	EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use crossterm::{ExecutableCommand, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::{Terminal, TerminalOptions, Viewport};

use crate::config::Config;
use crate::config::file::UiMode;

pub(crate) type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

fn terminal_supports_ui(stdout_is_tty: bool, term: Option<&str>) -> bool {
	if !stdout_is_tty {
		return false;
	}

	if term.is_some_and(|term| term.eq_ignore_ascii_case("dumb")) {
		return false;
	}

	true
}

fn mode_allows_enhanced_ui(mode: UiMode, stdout_is_tty: bool, term: Option<&str>) -> bool {
	!matches!(mode, UiMode::Plain) && terminal_supports_ui(stdout_is_tty, term)
}

fn mode_allows_fullscreen_ui(mode: UiMode, stdout_is_tty: bool, term: Option<&str>) -> bool {
	matches!(mode, UiMode::Tui) && terminal_supports_ui(stdout_is_tty, term)
}

pub(crate) fn use_enhanced_ui(config: &Config) -> bool {
	let term = env::var("TERM").ok();
	mode_allows_enhanced_ui(config.ui_mode(), stdout().is_terminal(), term.as_deref())
}

pub(crate) fn use_fullscreen_ui(config: &Config) -> bool {
	let term = env::var("TERM").ok();
	mode_allows_fullscreen_ui(config.ui_mode(), stdout().is_terminal(), term.as_deref())
}

pub(crate) fn poll_exit_event() -> Result<bool> {
	if !stdin().is_terminal() {
		return Ok(false);
	}

	if event::poll(Duration::from_millis(0))?
		&& let Event::Key(key) = event::read()?
		&& (KeyCode::Char('q') == key.code
			|| KeyCode::Char('c') == key.code && key.modifiers.contains(KeyModifiers::CONTROL))
	{
		return Ok(true);
	}
	Ok(false)
}

#[derive(Debug)]
struct RawModeGuard {
	active: bool,
}

impl RawModeGuard {
	pub(crate) fn new() -> Result<Self> {
		enable_raw_mode()?;
		Ok(Self { active: true })
	}

	pub(crate) fn disable(&mut self) -> Result<()> {
		if self.active {
			disable_raw_mode()?;
			self.active = false;
		}
		Ok(())
	}

	pub(crate) fn ensure_enabled(&mut self) -> Result<()> {
		if !self.active {
			enable_raw_mode()?;
			self.active = true;
		}
		Ok(())
	}
}

impl Drop for RawModeGuard {
	fn drop(&mut self) {
		if self.active {
			let _ = disable_raw_mode();
		}
	}
}

/// Owns an inline Ratatui viewport and the raw mode used while it is visible.
#[derive(Debug)]
pub(crate) struct InlineTerminalGuard {
	raw: RawModeGuard,
	terminal: Term,
	visible: bool,
}

impl InlineTerminalGuard {
	pub(crate) fn new(lines: u16) -> Result<Self> {
		let raw = RawModeGuard::new()?;
		let terminal = Terminal::with_options(
			CrosstermBackend::new(stdout()),
			TerminalOptions {
				viewport: Viewport::Inline(lines),
			},
		)?;

		Ok(Self {
			raw,
			terminal,
			visible: true,
		})
	}

	/// Returns the terminal used to draw inside the owned inline viewport.
	pub(crate) fn terminal_mut(&mut self) -> &mut Term { &mut self.terminal }

	/// Clears the viewport but keeps raw mode enabled for temporary output.
	pub(crate) fn hide(&mut self) -> Result<()> { self.leave(false) }

	/// Clears the viewport and disables raw mode before another UI takes
	/// control.
	pub(crate) fn suspend(&mut self) -> Result<()> { self.leave(true) }

	/// Re-enables raw mode and hides the cursor before inline rendering
	/// resumes.
	pub(crate) fn resume(&mut self) -> Result<()> {
		self.raw.ensure_enabled()?;
		self.terminal.hide_cursor()?;
		self.visible = true;
		Ok(())
	}

	fn leave(&mut self, disable_raw: bool) -> Result<()> {
		if !self.visible {
			return if disable_raw { self.raw.disable() } else { Ok(()) };
		}

		let terminal_result = self.restore_viewport();
		if terminal_result.is_ok() {
			self.visible = false;
		}
		let raw_result = if disable_raw { self.raw.disable() } else { Ok(()) };
		// Some terminal frontends retain Ratatui's last draw column even after
		// an absolute cursor move. Keep the carriage return as the final
		// handoff.
		let cursor_result = reset_cursor_column();

		terminal_result?;
		raw_result?;
		cursor_result
	}

	fn restore_viewport(&mut self) -> Result<()> {
		let origin = self.terminal.get_frame().area().as_position();
		self.terminal.clear()?;
		self.terminal.set_cursor_position(origin)?;
		self.terminal.show_cursor()?;
		Ok(())
	}
}

impl Drop for InlineTerminalGuard {
	fn drop(&mut self) { let _ = self.leave(true); }
}

fn reset_cursor_column() -> Result<()> {
	write!(stdout(), "\r")?;
	stdout().flush()?;
	Ok(())
}

#[derive(Debug)]
struct AltScreenGuard {
	active: bool,
}

impl AltScreenGuard {
	fn new() -> Result<Self> {
		stdout().execute(EnterAlternateScreen)?;
		Ok(Self { active: true })
	}

	fn leave(&mut self) -> Result<()> {
		if self.active {
			stdout().execute(LeaveAlternateScreen)?;
		}
		self.active = false;
		Ok(())
	}

	fn ensure_entered(&mut self, backend: &mut CrosstermBackend<std::io::Stdout>) -> Result<()> {
		if !self.active {
			execute!(backend, EnterAlternateScreen)?;
			self.active = true;
		}
		Ok(())
	}
}

impl Drop for AltScreenGuard {
	fn drop(&mut self) {
		if self.active {
			let _ = stdout().execute(LeaveAlternateScreen);
		}
	}
}

/// Owns a full-screen alternate-screen TUI, including mouse and raw mode.
#[derive(Debug)]
pub(crate) struct TerminalGuard {
	raw: RawModeGuard,
	alt: AltScreenGuard,
	mouse_enabled: bool,
	terminal: Term,
}

impl TerminalGuard {
	pub(crate) fn new() -> Result<Self> {
		let raw = RawModeGuard::new()?;
		let alt = match AltScreenGuard::new() {
			Ok(guard) => guard,
			Err(err) => {
				let _ = disable_raw_mode();
				return Err(err);
			},
		};

		let backend = CrosstermBackend::new(stdout());
		let terminal = match Term::new(backend) {
			Ok(term) => term,
			Err(err) => {
				let _ = stdout().execute(LeaveAlternateScreen);
				let _ = disable_raw_mode();
				return Err(err.into());
			},
		};

		Ok(Self {
			raw,
			alt,
			mouse_enabled: false,
			terminal,
		})
	}

	pub(crate) fn terminal_mut(&mut self) -> &mut Term { &mut self.terminal }

	pub(crate) fn enable_mouse_capture(&mut self) -> Result<()> {
		if !self.mouse_enabled {
			execute!(self.terminal.backend_mut(), EnableMouseCapture)?;
			self.mouse_enabled = true;
		}
		Ok(())
	}

	fn disable_mouse_capture(&mut self) -> Result<()> {
		if self.mouse_enabled {
			execute!(self.terminal.backend_mut(), DisableMouseCapture)?;
			self.mouse_enabled = false;
		}
		Ok(())
	}

	pub(crate) fn suspend(&mut self) -> Result<()> {
		self.terminal.show_cursor()?;

		if self.mouse_enabled {
			execute!(self.terminal.backend_mut(), DisableMouseCapture)?;
		}

		self.alt.leave()?;
		self.raw.disable()?;
		Ok(())
	}

	pub(crate) fn resume(&mut self) -> Result<()> {
		self.raw.ensure_enabled()?;
		self.alt.ensure_entered(self.terminal.backend_mut())?;

		if self.mouse_enabled {
			execute!(self.terminal.backend_mut(), EnableMouseCapture)?;
		}

		self.terminal.hide_cursor()?;
		self.terminal.clear()?;
		Ok(())
	}
}

impl Drop for TerminalGuard {
	fn drop(&mut self) {
		let _ = self.disable_mouse_capture();
		let _ = self.alt.leave();
		let _ = self.raw.disable();
		let _ = self.terminal.show_cursor();
	}
}

#[cfg(test)]
mod tests {
	use super::{mode_allows_enhanced_ui, mode_allows_fullscreen_ui};
	use crate::config::file::UiMode;

	#[test]
	fn plain_mode_disables_enhanced_ui() {
		assert!(!mode_allows_enhanced_ui(
			UiMode::Plain,
			true,
			Some("xterm-256color")
		));
		assert!(!mode_allows_fullscreen_ui(
			UiMode::Plain,
			true,
			Some("xterm-256color")
		));
	}

	#[test]
	fn non_tty_output_disables_ui() {
		assert!(!mode_allows_enhanced_ui(
			UiMode::Auto,
			false,
			Some("xterm-256color")
		));
		assert!(!mode_allows_fullscreen_ui(
			UiMode::Tui,
			false,
			Some("xterm-256color")
		));
	}

	#[test]
	fn dumb_term_disables_ui() {
		assert!(!mode_allows_enhanced_ui(UiMode::Auto, true, Some("dumb")));
		assert!(!mode_allows_fullscreen_ui(UiMode::Tui, true, Some("dumb")));
	}

	#[test]
	fn auto_mode_allows_only_enhanced_ui() {
		assert!(mode_allows_enhanced_ui(
			UiMode::Auto,
			true,
			Some("xterm-256color")
		));
		assert!(!mode_allows_fullscreen_ui(
			UiMode::Auto,
			true,
			Some("xterm-256color")
		));
	}

	#[test]
	fn tui_mode_allows_all_ui() {
		assert!(mode_allows_enhanced_ui(
			UiMode::Tui,
			true,
			Some("xterm-256color")
		));
		assert!(mode_allows_fullscreen_ui(
			UiMode::Tui,
			true,
			Some("xterm-256color")
		));
	}
}
