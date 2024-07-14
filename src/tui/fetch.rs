use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Flex, Layout, Rect};
use ratatui::style::{Style, Styled, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{
	Block, BorderType, List, ListItem, ListState, Padding, Paragraph, StatefulWidget, Widget,
};
use ratatui::Terminal;

use crate::colors::Theme;
use crate::config::Config;

struct FetchItem {
	url: String,
	score: String,
	selected: bool,
	style: Style,
	alt_style: Style,
}

impl FetchItem {
	fn to_list_items(&self) -> (ListItem, ListItem) {
		let (char, style) =
			if self.selected { ('✓', self.style) } else { ('☐', self.alt_style) };
		(
			ListItem::new(Line::styled(format!("{char} {}", self.url), style)),
			ListItem::new(Line::styled(&self.score, style)),
		)
	}
}

struct StatefulList {
	state: ListState,
	items: Vec<FetchItem>,
	align: (usize, usize),
	last_selected: Option<usize>,
}

impl StatefulList {
	fn new(config: &Config, scored: Vec<(String, u128)>) -> StatefulList {
		let mut items = vec![];
		let mut align = 0;
		let mut score_align = 0;

		let style = config.rat_style(Theme::Primary);
		let alt_style = config.rat_style(Theme::Regular);

		for (url, u_score) in scored {
			// Calculate alignment
			if url.len() > align {
				align = url.len();
			}
			let score = format!("{u_score} ms");
			if score.len() > score_align {
				score_align = score.len()
			}

			items.push(FetchItem {
				url,
				score,
				selected: false,
				style,
				alt_style,
			});
		}

		StatefulList {
			state: ListState::default(),
			align: (align, score_align),
			items,
			last_selected: None,
		}
	}

	fn next(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i >= self.items.len() - 1 {
					0
				} else {
					i + 1
				}
			},
			None => self.last_selected.unwrap_or(0),
		};
		self.state.select(Some(i));
	}

	fn previous(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i == 0 {
					self.items.len() - 1
				} else {
					i - 1
				}
			},
			None => self.last_selected.unwrap_or(0),
		};
		self.state.select(Some(i));
	}
}

/// The Struct that drives the Fetch TUI
pub struct App<'a> {
	config: &'a Config,
	items: StatefulList,
}

impl<'a> App<'a> {
	pub fn new(config: &'a Config, scored: Vec<(String, u128)>) -> Self {
		App {
			config,
			items: StatefulList::new(config, scored),
		}
	}

	/// Changes the status of the selected list item
	fn change_status(&mut self) {
		if let Some(i) = self.items.state.selected() {
			self.items.items[i].selected = match self.items.items[i].selected {
				true => false,
				false => true,
			}
		}
	}

	fn go_top(&mut self) { self.items.state.select(Some(0)); }

	fn go_bottom(&mut self) { self.items.state.select(Some(self.items.items.len() - 1)); }

	pub fn run(mut self, mut terminal: Terminal<impl Backend>) -> Result<Vec<String>> {
		loop {
			self.draw(&mut terminal)?;

			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					use KeyCode::*;
					match key.code {
						Char('q') | Enter => {
							// Return only the selected Urls.
							return Ok(self
								.items
								.items
								.into_iter()
								.filter(|f| f.selected)
								.map(|f| f.url)
								.collect());
						},
						// CTRL+C will return an empty vec to exit cleanly without progressing.
						Char('c') => {
							if key.modifiers.contains(KeyModifiers::CONTROL) {
								return Ok(vec![]);
							}
						},
						Char('j') | Down => self.items.next(),
						Char('k') | Up => self.items.previous(),
						Char(' ') => self.change_status(),
						Char('g') | Home => self.go_top(),
						Char('G') | End => self.go_bottom(),
						_ => {},
					}
				}
			}
		}
	}

	fn draw(&mut self, terminal: &mut Terminal<impl Backend>) -> Result<()> {
		terminal.draw(|f| f.render_widget(self, f.size()))?;
		Ok(())
	}

	fn render_lists(&mut self, area: Rect, buf: &mut Buffer) {
		let header = format!("  {}  ", "Nala Fetch");

		let outer_block = Block::bordered()
			.title(header.set_style(self.config.rat_style(Theme::Highlight)))
			.title_alignment(Alignment::Center)
			.border_type(BorderType::Rounded)
			.style(self.config.rat_style(Theme::Primary));

		let [mirror_area, score_area] = Layout::horizontal([
			Constraint::Length(self.items.align.0 as u16 + 4),
			Constraint::Length(self.items.align.1 as u16),
		])
		.flex(Flex::Center)
		.areas(outer_block.inner(area));

		outer_block.render(area, buf);

		let mut mirror_items: Vec<ListItem> = vec![];
		let mut score_items: Vec<ListItem> = vec![];

		for fetch_item in &self.items.items {
			let item = fetch_item.to_list_items();

			mirror_items.push(item.0);

			score_items.push(item.1);
		}

		let highlight = self.config.rat_style(Theme::Secondary).reversed();
		let block = self.config.rat_style(Theme::Regular);

		StatefulWidget::render(
			List::new(mirror_items)
				.block(fetch_block(block, "Mirrors:"))
				.highlight_style(highlight),
			mirror_area,
			buf,
			&mut self.items.state,
		);
		StatefulWidget::render(
			List::new(score_items)
				.block(fetch_block(block, "Score:"))
				.highlight_style(highlight),
			score_area,
			buf,
			&mut self.items.state,
		);
	}
}

impl<'a> Widget for &mut App<'a> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// Create a space for header, todo list and the footer.
		let [list_area, info_area, footer_area] = Layout::vertical([
			Constraint::Min(0),
			Constraint::Length(2),
			Constraint::Length(2),
		])
		.areas(area);

		self.render_lists(list_area, buf);

		Paragraph::new("\nScore is how many milliseconds it takes to download the Release file.")
			.centered()
			.italic()
			.render(info_area, buf);

		Paragraph::new(
			"\nUse ↓↑ to move, Space to select/unselect, Home/End to go top/bottom, q/Enter to \
			 exit.",
		)
		.centered()
		.render(footer_area, buf);
	}
}

fn fetch_block(style: Style, title: &str) -> Block {
	Block::default()
		.title(title)
		.style(style)
		.padding(Padding::vertical(1))
}
