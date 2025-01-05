use std::collections::{BTreeMap, HashMap};
use std::{fmt, io};

use ansi_to_tui::IntoText;
use anyhow::Result;
use crossterm::event::{
	self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Max, Min};
use ratatui::layout::{Alignment, Flex, Layout, Margin, Rect};
use ratatui::prelude::CrosstermBackend;
use ratatui::style::{Style, Styled};
use ratatui::text::Text;
use ratatui::widgets::{
	Block, BorderType, Cell, HighlightSpacing, Padding, Paragraph, Row, Scrollbar,
	ScrollbarOrientation, ScrollbarState, StatefulWidget, Table, TableState, Tabs, Widget, Wrap,
};
use ratatui::Terminal;
use rust_apt::util::DiskSpace;
use rust_apt::Cache;

use super::Term;
use crate::cmd::{HistoryPackage, Operation};
use crate::config::{Config, Theme};

#[derive(Debug)]
pub struct Item {
	align: Alignment,
	style: Style,
	pub string: String,
}

impl Item {
	fn new(align: Alignment, style: Style, string: String) -> Self {
		Self {
			align,
			style,
			string,
		}
	}

	pub fn center(style: Style, string: String) -> Self {
		Self::new(Alignment::Center, style, string)
	}

	pub fn right(style: Style, string: String) -> Self {
		Self::new(Alignment::Right, style, string)
	}

	pub fn left(style: Style, string: String) -> Self { Self::new(Alignment::Left, style, string) }

	fn get_cell(&self) -> Cell {
		Cell::from(
			self.string
				.into_text()
				.unwrap()
				.style(self.style)
				.alignment(self.align),
		)
	}
}

impl fmt::Display for Item {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(&self.string) }
}

pub struct App<'a> {
	state: TableState,
	scroll_state: ScrollbarState,
	config: &'a Config,
	items: &'a Vec<HistoryPackage>,
}

impl<'a> App<'a> {
	fn new(config: &'a Config, items: &'a Vec<HistoryPackage>) -> Self {
		let scroll_state = ScrollbarState::new(items.len() - 1);
		Self {
			state: TableState::default().with_selected(0),
			scroll_state,
			config,
			items,
		}
	}

	fn set_state(&mut self, i: usize) {
		self.state.select(Some(i));
		self.scroll_state = self.scroll_state.position(i);
	}

	pub fn home(&mut self) { self.set_state(0); }

	pub fn end(&mut self) { self.set_state(self.items.len() - 1); }

	pub fn next(&mut self) {
		let i = self.state.selected().unwrap_or_default();
		if i >= self.items.len() - 1 {
			return;
		}
		self.set_state(i + 1);
	}

	pub fn previous(&mut self) {
		let i = self.state.selected().unwrap_or_default();
		if i == 0 {
			return;
		}
		self.set_state(i - 1);
	}

	fn render_table(&mut self, area: Rect, buf: &mut Buffer) {
		let highlight = self.config.rat_style(Theme::Primary);
		let white = self.config.rat_style(Theme::Regular);

		// Choose which headers based on the inner items of the SummaryPkg
		let headers = if self.items[0].items(self.config).len() > 3 {
			vec!["Package:", "Old Version:", "New Version:", "Size:"]
		} else {
			vec!["Package:", "Version:", "Size:"]
		};
		// Get max length of the headers incase they are the longest in the columns
		let header_max = headers.iter().map(|h| h.len()).max().unwrap_or_default();

		// Build the headers into Cells
		let header = headers
			.into_iter()
			.zip(self.items[0].items(self.config).iter())
			.map(|(str, i)| Cell::from(Text::from(str).alignment(i.align)))
			.collect::<Row>()
			.style(white);

		let mut constraints = vec![];
		for i in 0..self.items[0].items(self.config).len() {
			constraints.push(
				self.items
					.iter()
					.map(|item| item.items(self.config)[i].string.len().max(header_max))
					.max()
					.unwrap_or_default() as u16,
			)
		}

		let t = Table::new(
			self.items.iter().map(|vec| {
				Row::from_iter(vec.items(self.config).iter().map(|item| item.get_cell()))
			}),
			constraints,
		)
		.header(header)
		.row_highlight_style(highlight)
		.flex(Flex::SpaceAround)
		.block(basic_block(self.config))
		.highlight_spacing(HighlightSpacing::Never);

		StatefulWidget::render(t, area, buf, &mut self.state);
	}
}

impl StatefulWidget for &mut App<'_> {
	type State = u8;

	fn render(self, area: Rect, buf: &mut Buffer, _: &mut Self::State) {
		let table_area = Layout::horizontal([Min(0), Length(3)]).split(area);

		self.render_table(table_area[0], buf);

		basic_block(self.config).render(table_area[1], buf);

		StatefulWidget::render(
			Scrollbar::default()
				.orientation(ScrollbarOrientation::VerticalRight)
				.thumb_style(self.config.rat_style(Theme::Primary))
				.track_style(self.config.rat_style(Theme::Secondary))
				.begin_symbol(None)
				.end_symbol(None),
			table_area[1].inner(Margin {
				vertical: 1,
				horizontal: 1,
			}),
			buf,
			&mut self.scroll_state,
		);
	}
}

pub struct SummaryTab<'a> {
	cache: &'a Cache,
	config: &'a Config,
	pkg_set: BTreeMap<Operation, App<'a>>,
	// Array first is the header, second is string.
	download_size: Option<Vec<String>>,
	disk_space: Vec<String>,
	i: usize,
	tabs: Vec<Operation>,
}

impl<'a> SummaryTab<'a> {
	pub fn new(
		cache: &'a Cache,
		config: &'a Config,
		pkg_set: &'a HashMap<Operation, Vec<HistoryPackage>>,
	) -> Self {
		let pkg_set = pkg_set
			.iter()
			.map(|(op, set)| (*op, App::new(config, set)))
			.collect();

		let size = cache.depcache().download_size();
		let download_size = if size > 0 {
			Some(vec![
				"Total download size:".to_string(),
				config.unit_str(size),
			])
		} else {
			None
		};

		let disk_space = match cache.depcache().disk_size() {
			DiskSpace::Require(num) => {
				vec!["Disk space required:".to_string(), config.unit_str(num)]
			},
			DiskSpace::Free(num) => vec!["Disk space to free:".to_string(), config.unit_str(num)],
		};

		let mut tabs = Self {
			cache,
			config,
			pkg_set,
			download_size,
			disk_space,
			i: 0,
			tabs: Operation::to_vec(),
		};

		for (i, tab) in tabs.tabs.iter().enumerate() {
			if tabs.pkg_set.contains_key(tab) {
				tabs.i = i;
				break;
			}
		}

		tabs
	}

	pub fn current_tab(&self) -> Operation { self.tabs[self.i] }

	pub fn current(&self) -> &App<'a> { self.pkg_set.get(&self.current_tab()).unwrap() }

	pub fn current_mut(&mut self) -> &mut App<'a> {
		self.pkg_set.get_mut(&self.current_tab()).unwrap()
	}

	fn real_next(&mut self) -> bool {
		let max = self.tabs.len() - 1;
		if self.i >= max {
			self.i = max;
			return false;
		}
		self.i += 1;
		true
	}

	fn real_previous(&mut self) -> bool {
		if self.i == 0 {
			return false;
		}
		self.i -= 1;
		true
	}

	pub fn next_tab(&mut self) {
		let i = self.i;
		self.real_next();
		while !self.pkg_set.contains_key(&self.tabs[self.i]) {
			if !self.real_next() {
				self.i = i;
				return;
			}
		}
	}

	pub fn previous_tab(&mut self) {
		let i = self.i;
		self.real_previous();
		while !self.pkg_set.contains_key(&self.tabs[self.i]) {
			if !self.real_previous() {
				self.i = i;
				return;
			}
		}
	}

	fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
		let titles: Vec<&str> = self
			.tabs
			.iter()
			.filter_map(
				|op| {
					if self.pkg_set.contains_key(op) {
						Some(op.as_str())
					} else {
						None
					}
				},
			)
			.collect();

		let tab_size = titles.iter().map(|s| s.len()).sum::<usize>() + titles.len() + 1;
		let new_area = Layout::horizontal([tab_size as u16]).split(area);

		let position = self
			.pkg_set
			.iter()
			.position(|(op, _)| op == &self.current_tab())
			.unwrap();

		Tabs::new(titles)
			.highlight_style(self.config.rat_style(Theme::Primary))
			.select(position)
			.padding("", "")
			.divider(" ")
			.render(new_area[0], buf);
	}

	pub async fn run(&mut self) -> Result<bool> {
		enable_raw_mode()?;
		let mut stdout = io::stdout();
		execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
		let backend = CrosstermBackend::new(stdout);
		let mut terminal = Terminal::new(backend)?;

		loop {
			terminal
				.draw(|frame| frame.render_stateful_widget(&mut *self, frame.area(), &mut 0))?;

			match event::read()? {
				Event::Key(key) => {
					if key.kind == KeyEventKind::Press {
						match key.code {
							KeyCode::Char('q') | KeyCode::Esc => {
								restore_terminal(&mut terminal)?;
								return Ok(false);
							},
							KeyCode::Char('y') => {
								restore_terminal(&mut terminal)?;
								return Ok(true);
							},
							KeyCode::Char('l') | KeyCode::Right => self.next_tab(),
							KeyCode::Char('h') | KeyCode::Left => self.previous_tab(),
							KeyCode::Char('j') | KeyCode::Down => self.current_mut().next(),
							KeyCode::Char('k') | KeyCode::Up => self.current_mut().previous(),
							KeyCode::Home => self.current_mut().home(),
							KeyCode::End => self.current_mut().end(),
							KeyCode::PageDown => {
								for _ in 0..10 {
									self.current_mut().next();
								}
							},
							KeyCode::PageUp => {
								for _ in 0..10 {
									self.current_mut().previous();
								}
							},
							KeyCode::Enter => {
								let app = self.current();
								if let Some(i) = app.state.selected() {
									app.items[i]
										.render_changelog(self.cache, &mut terminal)
										.await?;
								}
							},
							KeyCode::Char('s') => {
								let app = self.current();
								if let Some(i) = app.state.selected() {
									app.items[i].render_show(
										self.cache,
										self.config,
										&mut terminal,
									)?;
								}
							},
							_ => {},
						}
					}
				},
				Event::Mouse(event) => match event.kind {
					MouseEventKind::ScrollDown => self.current_mut().next(),
					MouseEventKind::ScrollUp => self.current_mut().previous(),
					_ => {},
				},
				_ => {},
			}
		}
	}
}

impl StatefulWidget for &mut SummaryTab<'_> {
	type State = u8;

	fn render(self, area: Rect, buf: &mut Buffer, _: &mut Self::State) {
		let block = header_block(self.config, "Nala Upgrade");

		let mut summary = vec![];
		for op in Operation::to_vec().iter() {
			let Some(set) = self.pkg_set.get(op) else {
				continue;
			};

			summary.push(vec![format!("{op}:"), format!("{} Pkgs", set.items.len())]);
		}

		summary.push(vec![]);
		if let Some(array) = &self.download_size {
			summary.push(array.clone());
		}
		summary.push(self.disk_space.clone());

		let mut header_len = 0;
		let mut size_len = 0;
		for vec in &summary {
			if vec.is_empty() {
				continue;
			}
			if vec[0].len() > header_len {
				header_len = vec[0].len()
			}
			if vec[1].len() > size_len {
				size_len = vec[1].len()
			}
		}

		let [tab, table, footer] =
			Layout::vertical([Length(1), Min(0), Length(summary.len() as u16)])
				.flex(Flex::Center)
				.areas(block.inner(area));

		block.render(area, buf);

		self.render_tabs(tab, buf);

		self.pkg_set
			.get_mut(&self.current_tab())
			.unwrap()
			.render(table, buf, &mut 0);

		let text = [
			"(↑) move up | (↓) move down",
			"(→) next tab | (←) previous tab",
			"(Enter) show changelog | (s) show version info",
			"(q) quit | (y) start upgrade",
		];

		let [summary_area, info_area] = Layout::horizontal([
			Max((header_len + size_len) as u16),
			Max(text.iter().map(|s| s.len()).max().unwrap_or_default() as u16),
		])
		.flex(Flex::SpaceAround)
		.areas(footer);

		let t = Table::new(
			summary.iter().map(|a| {
				if a.is_empty() {
					Row::new(a.clone())
				} else {
					Row::new([
						Cell::from(Text::from(a[0].as_str())),
						Cell::from(Text::from(a[1].as_str()).right_aligned()),
					])
				}
			}),
			[Length(header_len as u16), Length(size_len as u16)],
		);
		Widget::render(t, summary_area, buf);

		Paragraph::new(Text::from_iter(text))
			.centered()
			.style(self.config.rat_style(Theme::Secondary))
			.wrap(Wrap::default())
			.render(info_area, buf);
	}
}

/// Restore the terminal
pub fn restore_terminal(terminal: &mut Term) -> Result<()> {
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;
	Ok(())
}

pub fn header_block<'a>(config: &'a Config, title: &'a str) -> Block<'a> {
	basic_block(config)
		.title(format!("  {title}  ").set_style(config.rat_style(Theme::Highlight)))
		.title_alignment(Alignment::Center)
		.padding(Padding::horizontal(1))
}

pub fn basic_block(config: &Config) -> Block {
	Block::bordered()
		.border_type(BorderType::Thick)
		.border_style(config.rat_style(Theme::Primary))
}
