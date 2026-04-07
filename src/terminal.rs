//! Terminal runtime helpers.
//!
//! This module owns terminal/session mechanics such as TUI mode policy,
//! raw mode, alternate screen, mouse capture, and the shared terminal type.

use std::env;
use std::io::{stdout, IsTerminal};
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
	self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers,
};
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::config::file::UiMode;
use crate::config::Config;

pub(crate) type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

fn mode_allows_tui(mode: UiMode, stdout_is_tty: bool, term: Option<&str>) -> bool {
	if matches!(mode, UiMode::Plain) {
		return false;
	}

	if !stdout_is_tty {
		return false;
	}

	if term.is_some_and(|term| term.eq_ignore_ascii_case("dumb")) {
		return false;
	}

	true
}

pub(crate) fn use_tui(config: &Config) -> bool {
	let term = env::var("TERM").ok();
	mode_allows_tui(config.ui_mode(), stdout().is_terminal(), term.as_deref())
}

pub(crate) fn poll_exit_event() -> Result<bool> {
	if event::poll(Duration::from_millis(0))? {
		if let Event::Key(key) = event::read()? {
			if KeyCode::Char('q') == key.code {
				return Ok(true);
			}

			if KeyCode::Char('c') == key.code && key.modifiers.contains(KeyModifiers::CONTROL) {
				return Ok(true);
			}
		}
	}
	Ok(false)
}

#[derive(Debug)]
pub(crate) struct RawModeGuard {
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

	fn ensure_enabled(&mut self) -> Result<()> {
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
	use super::mode_allows_tui;
	use crate::config::file::UiMode;

	#[test]
	fn plain_mode_disables_tui() {
		assert!(!mode_allows_tui(
			UiMode::Plain,
			true,
			Some("xterm-256color")
		));
	}

	#[test]
	fn non_tty_output_disables_tui() {
		assert!(!mode_allows_tui(
			UiMode::Auto,
			false,
			Some("xterm-256color")
		));
		assert!(!mode_allows_tui(UiMode::Tui, false, Some("xterm-256color")));
	}

	#[test]
	fn dumb_term_disables_tui() {
		assert!(!mode_allows_tui(UiMode::Auto, true, Some("dumb")));
		assert!(!mode_allows_tui(UiMode::Tui, true, Some("dumb")));
	}

	#[test]
	fn supported_terminal_allows_tui() {
		assert!(mode_allows_tui(UiMode::Auto, true, Some("xterm-256color")));
		assert!(mode_allows_tui(UiMode::Tui, true, Some("xterm-256color")));
	}
}
