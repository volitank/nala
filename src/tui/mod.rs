use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;

pub mod fetch;
pub mod progress;
pub mod summary;

pub use progress::NalaProgressBar;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

pub type Term = Terminal<CrosstermBackend<std::io::Stdout>>;

pub fn poll_exit_event() -> Result<bool> {
	if crossterm::event::poll(Duration::from_millis(0))? {
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

pub fn init_terminal() -> Result<Term> {
	enable_raw_mode()?;
	let mut backend = CrosstermBackend::new(std::io::stdout());
	backend.execute(EnterAlternateScreen)?;
	Ok(Terminal::new(backend)?)
}

pub fn restore_terminal() -> Result<()> {
	disable_raw_mode()?;
	std::io::stdout().execute(LeaveAlternateScreen)?;
	Ok(())
}
