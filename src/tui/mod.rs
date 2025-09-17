use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{execute, ExecutableCommand};

pub mod fetch;
pub mod progress;
pub mod summary;

pub use progress::{NalaProgressBar, UnitStr};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::widgets::{Block, BorderType, Padding, Paragraph};
use ratatui::{CompletedFrame, Frame, Terminal, TerminalOptions, Viewport};

use crate::config::color::Color;
use crate::config::{Config, Theme};

type CrossTerm = Terminal<CrosstermBackend<std::io::Stdout>>;

pub struct Term {
	pub term: CrossTerm,
}

impl DerefMut for Term {
	fn deref_mut(&mut self) -> &mut Self::Target { &mut self.term }
}

impl Deref for Term {
	type Target = CrossTerm;

	fn deref(&self) -> &Self::Target { &self.term }
}

impl Term {
	pub fn new() -> Result<Term> {
		Ok(Term {
			term: init_terminal()?,
		})
	}

	pub fn init_viewport(size: u16) -> Result<Term> {
		let term = Terminal::with_options(
			init_backend()?,
			TerminalOptions {
				viewport: Viewport::Inline(size),
			},
		)?;

		Ok(Self { term })
	}

	/// Restore the terminal
	pub fn restore(&mut self) -> Result<()> {
		disable_raw_mode()?;
		execute!(
			self.backend_mut(),
			LeaveAlternateScreen,
			DisableMouseCapture
		)?;
		self.show_cursor()?;
		crate::config::logger::DISABLED.swap(false, Ordering::Relaxed);
		Ok(())
	}

	pub fn draw(
		&mut self,
		config: &Config,
		items: &[&dyn Drawable],
	) -> anyhow::Result<CompletedFrame<'_>> {
		let frame = self.term.draw(|f| {
			if items.is_empty() {
				return;
			}

			let block = crate::tui::vblock(&config.color);
			// Compute inner area before moving `block` into render_widget
			let inner = block.inner(f.area());
			// Draw the border
			f.render_widget(block, f.area());

			// If there is more than one item, treat the last as the progress bar
			let (body_items, progress_item) = if items.len() > 1 {
				(&items[..items.len() - 1], Some(items[items.len() - 1]))
			} else {
				(&items[..], None)
			};

			if let Some(pb) = progress_item {
				// Split inner into [body][progress=1 row]
				let outer_chunks = ratatui::layout::Layout::default()
					.direction(ratatui::layout::Direction::Vertical)
					.constraints([
						ratatui::layout::Constraint::Min(0),
						ratatui::layout::Constraint::Length(1),
					])
					.split(inner);

				let body_area = outer_chunks[0];
				let progress_area = outer_chunks[1];

				// Now split the body among the non-progress items
				if !body_items.is_empty() {
					let n = body_items.len() as u32;
					let body_chunks = ratatui::layout::Layout::default()
						.direction(ratatui::layout::Direction::Vertical)
						.constraints(vec![ratatui::layout::Constraint::Ratio(1, n); n as usize])
						.split(body_area);

					for (w, area) in body_items.iter().zip(body_chunks.into_iter()) {
						w.draw(f, *area);
					}
				}

				// Progress bar anchored to the bottom row
				pb.draw(f, progress_area);
			} else {
				// No progress bar, split inner equally among all items
				let n = items.len() as u32;
				let chunks = ratatui::layout::Layout::default()
					.direction(ratatui::layout::Direction::Vertical)
					.constraints(vec![ratatui::layout::Constraint::Ratio(1, n); n as usize])
					.split(inner);

				for (w, area) in items.iter().zip(chunks.into_iter()) {
					w.draw(f, *area);
				}
			}
		})?;

		Ok(frame)
	}
}

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

fn init_backend() -> Result<CrosstermBackend<std::io::Stdout>> {
	crate::config::logger::DISABLED.swap(true, Ordering::Relaxed);
	enable_raw_mode()?;
	Ok(CrosstermBackend::new(std::io::stdout()))
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
	let mut term = Terminal::new(init_backend()?)?;
	term.backend_mut().execute(EnterAlternateScreen)?;
	Ok(term)
}

pub fn vblock(color: &Color) -> Block<'static> {
	Block::bordered()
		.border_type(BorderType::Rounded)
		.padding(Padding::horizontal(1))
		.style(color.rat_style(Theme::Primary))
}

pub fn paragraph(text: &str) -> Paragraph { Paragraph::new(text).right_aligned() }

pub trait Drawable {
	fn draw(&self, f: &mut Frame, area: Rect);
	// Optional: let a widget hint a fixed height in rows
	fn height_hint(&self) -> Option<u16> { None }
}
