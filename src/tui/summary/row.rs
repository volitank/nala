use std::cell::OnceCell;
use std::io::Write;
use std::process::Stdio;
use std::{fmt, io};

use ansi_to_tui::IntoText;
use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::layout::Constraint::Length;
use ratatui::layout::{Alignment, Layout};
use ratatui::style::Style;
use ratatui::text::Text;
use ratatui::widgets::{Cell, Paragraph, Wrap};
use rust_apt::Cache;
use tokio::sync::OnceCell as AsyncOnceCell;

use crate::config::{color, Config, Theme};
use crate::libnala::PackageTransition;
use crate::terminal::TerminalGuard;
use crate::tui::{style as tui_style, summary};
use crate::util;

#[derive(Debug)]
pub struct Item {
	pub(crate) align: Alignment,
	style: Style,
	pub(crate) string: String,
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

	pub(crate) fn get_cell(&self) -> Cell<'_> {
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

pub struct SummaryRow<'a> {
	package: &'a PackageTransition,
	items: OnceCell<Vec<Item>>,
	changelog: AsyncOnceCell<String>,
}

impl<'a> SummaryRow<'a> {
	pub fn new(package: &'a PackageTransition) -> Self {
		Self {
			package,
			items: OnceCell::new(),
			changelog: AsyncOnceCell::new(),
		}
	}

	fn display_version(&self) -> Option<&str> {
		self.package
			.after
			.version_str()
			.or(self.package.before.version_str())
	}

	fn display_old_version(&self) -> Option<&str> {
		match (
			self.package.before.version_str(),
			self.package.after.version_str(),
		) {
			(Some(before), Some(after)) if before != after => Some(before),
			_ => None,
		}
	}

	pub(crate) fn headers(&self) -> Vec<&'static str> {
		match (
			self.display_old_version().is_some(),
			self.package.held_reason.is_some(),
		) {
			(true, true) => vec![
				"Package:",
				"Old Version:",
				"New Version:",
				"Reason:",
				"Size:",
			],
			(true, false) => vec!["Package:", "Old Version:", "New Version:", "Size:"],
			(false, true) => vec!["Package:", "Version:", "Reason:", "Size:"],
			(false, false) => vec!["Package:", "Version:", "Size:"],
		}
	}

	pub fn items(&self, config: &Config) -> &Vec<Item> {
		self.items.get_or_init(|| {
			let secondary = tui_style::style(config, self.package.operation);
			let primary = tui_style::style(config, Theme::Regular);

			let colored = color::color!(self.package.operation, &self.package.name).to_string();
			let mut items = vec![Item::left(secondary, colored)];

			if let Some(old) = self.display_old_version() {
				items.push(Item::center(primary, old.to_string()));
				let new_version = self.display_version().unwrap_or("Unknown").to_string();
				items.push(Item::center(primary, util::version_diff(old, new_version)));
			} else {
				items.push(Item::center(
					primary,
					self.display_version().unwrap_or("Unknown").to_string(),
				));
			}
			if let Some(reason) = &self.package.held_reason {
				items.push(Item::center(primary, reason.summary()));
			}
			items.push(Item::right(primary, config.unit_str(self.package.size)));
			items
		})
	}

	/// Lazily fetches and caches the changelog for the package version shown by
	/// this row.
	pub async fn get_changelog(&self, cache: &Cache) -> Result<&String> {
		self.changelog
			.get_or_try_init(|| async {
				let uri = match self.package.get_pkg(cache)?.changelog_uri() {
					Some(uri) => uri,
					None => bail!("Unable to find Changelog URI"),
				};

				Ok(reqwest::get(uri).await?.error_for_status()?.text().await?)
			})
			.await
	}

	/// Opens the package changelog in a pager while temporarily suspending the
	/// TUI.
	pub(crate) async fn render_changelog(
		&self,
		cache: &Cache,
		terminal: &mut TerminalGuard,
	) -> Result<()> {
		let changelog = match self.get_changelog(cache).await {
			Ok(log) => log,
			Err(e) => &format!("{e:?}"),
		};

		terminal.suspend()?;

		let result: Result<()> = (|| {
			let mut pager = std::process::Command::new("less")
				.arg("--raw-control-chars")
				.arg("--clear-screen")
				.stdin(Stdio::piped())
				.spawn()?;

			if let Some(stdin) = pager.stdin.as_mut() {
				if let Err(err) = stdin.write_all(changelog.as_bytes()) {
					match err.kind() {
						io::ErrorKind::BrokenPipe => {},
						_ => return Err(err.into()),
					}
				}
			}

			pager.wait()?;
			Ok(())
		})();

		terminal.resume()?;
		result
	}

	/// Renders the package metadata view used by interactive summary
	/// inspection.
	pub(crate) fn render_show(
		&self,
		cache: &Cache,
		config: &Config,
		terminal: &mut TerminalGuard,
	) -> Result<()> {
		let show = crate::cmd::ShowVersion::new(self.package.get_version(cache)?);
		let terminal = terminal.terminal_mut();
		terminal.clear()?;

		let mut lines: Vec<Text> = vec![];
		for (head, info) in show.pretty_map() {
			let mut split = info.split('\n');
			if let Some(first) = split.next() {
				lines.push(format!("{}: {first}", color::highlight!(head)).into_text()?);
				for line in split {
					lines.push(line.to_string().into_text()?);
				}
			}
		}

		loop {
			terminal.draw(|f| {
				let block = summary::header_block(config, "Nala Upgrade");
				let inner = block.inner(f.area());
				let constraints = lines
					.iter()
					.map(|line| Length((line.width() as f32 / inner.width as f32).ceil() as u16))
					.collect::<Vec<_>>();
				let layout = Layout::vertical(constraints).split(block.inner(f.area()));

				f.render_widget(block, f.area());
				for (i, line) in lines.iter().enumerate() {
					f.render_widget(
						Paragraph::new(line.clone()).wrap(Wrap::default()),
						layout[i],
					)
				}
			})?;

			if let Event::Key(key) = event::read()? {
				if key.kind == KeyEventKind::Press {
					match key.code {
						KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
						_ => {},
					}
				}
			}
		}
	}
}
