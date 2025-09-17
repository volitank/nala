use std::io::Write;

use ansi_to_tui::IntoText;
use anyhow::{anyhow, bail, Context, Result};
use chrono::format::{DelayedFormat, StrftimeItems};
use chrono::{DateTime, Local, Utc};
use crossterm::event::{self, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use indexmap::IndexMap;
use ratatui::layout::Constraint::Length;
use ratatui::layout::Layout;
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Wrap};
use rust_apt::{Cache, Package, Version};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use super::system::get_user;
use super::Operation;
use crate::config::{color, Config, Paths, Theme};
use crate::fs::AsyncFs;
use crate::libnala::ShowVersion;
use crate::{table, tui};

pub struct HistoryFile(Vec<HistoryEntry>);

impl HistoryFile {
	pub async fn from_config(config: &Config) -> Result<HistoryFile> {
		Ok(HistoryFile(get_history(config).await?))
	}

	pub fn get(&self, id: &str) -> Result<&HistoryEntry> {
		let id = if id == "last" {
			self.0.len()
		} else {
			id.parse::<usize>().with_context(|| {
				format!("'{id}' is not valid. Use 'last' or the number of the entry.")
			})?
		};

		if let Some(entry) = self.iter().nth(id - 1) {
			return Ok(entry);
		};

		bail!("History entry with ID '{id}' does not exist")
	}

	pub fn iter(&self) -> impl Iterator<Item = &HistoryEntry> { self.0.iter() }

	pub fn table(&self) -> comfy_table::Table {
		table::get_table(&["ID", "Command", "Date and Time", "Requested-By", "Altered"])
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
	pub id: u32,
	pub date: String,
	pub requested_by: String,
	pub command: String,
	pub altered: usize,
	packages: Vec<HistoryPackage>,
}

// Package: zstd
// Status: install ok installed
// Priority: optional
// Section: utils
// Installed-Size: 2247
// Maintainer: RPM packaging team <team+pkg-rpm@tracker.debian.org>
// Architecture: amd64
// Multi-Arch: foreign
// Source: libzstd
// Version: 1.5.6+dfsg-1
// Depends: libc6 (>= 2.34), libgcc-s1 (>= 3.0), liblz4-1 (>= 1.8.0), liblzma5
// (>= 5.1.1alpha+20120614), libstdc++6 (>= 12), zlib1g (>= 1:1.1.4)
// Description: fast lossless compression algorithm -- CLI tool
//  Zstd, short for Zstandard, is a fast lossless compression algorithm,
// targeting  real-time compression scenarios at zlib-level compression ratio.
//  .
//  This package contains the CLI program implementing zstd.
// Homepage: https://github.com/facebook/zstd

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryPackage {
	pub name: String,
	pub version: String,
	pub old_version: Option<String>,
	pub size: u64,
	pub operation: Operation,
	pub auto_installed: bool,
	#[serde(skip)]
	items: std::cell::OnceCell<Vec<String>>,
	#[serde(skip)]
	changelog: OnceCell<String>,
}

impl HistoryEntry {
	pub fn new(id: u32, date: String, packages: Vec<HistoryPackage>) -> Self {
		let (uid, username) = get_user();
		Self {
			id,
			date,
			requested_by: format!("{username} ({uid})"),
			command: std::env::args().skip(1).collect::<Vec<String>>().join(" "),
			altered: packages.len(),
			packages,
		}
	}

	pub fn pkgs(&self) -> &Vec<HistoryPackage> { &self.packages }

	pub fn to_map(&self) -> IndexMap<Operation, Vec<&HistoryPackage>> {
		let mut map: IndexMap<Operation, Vec<&HistoryPackage>> = IndexMap::new();
		for op in Operation::to_vec() {
			let pkgs = self
				.packages
				.as_slice()
				.iter()
				.filter(|p| p.operation == op)
				.collect::<Vec<_>>();

			if pkgs.is_empty() {
				continue;
			}

			map.insert(op, pkgs);
		}
		map
	}

	pub async fn write_to_file(&self, config: &Config) -> Result<()> {
		let mut filename = config.get_path(&Paths::History);
		filename.push(format!("{}.bin", self.id));

		let data = serde_json::to_string_pretty(self)
			// let data = bincode::serialize(&self)
			.with_context(|| format!("Unable to serialize HistoryEntry\n\n    {self:?}"))?;

		filename.write(data).await
	}

	pub fn date(&self) -> Option<DelayedFormat<StrftimeItems<'_>>> {
		Some(
			self.date
				.parse::<DateTime<Utc>>()
				.ok()?
				.with_timezone(&Local)
				.format("%Y-%m-%d %H:%M:%S %Z"),
		)
	}

	pub fn as_row(&self) -> comfy_table::Row {
		let display: &[&dyn std::fmt::Display; 5] = &[
			&self.id,
			&self.command,
			&self.date().unwrap(),
			&self.requested_by,
			&self.altered,
		];
		// display.iter().map(|f| f.to_string()).collect();
		comfy_table::Row::from(display)
	}
}

impl HistoryPackage {
	pub fn from_version(
		operation: Operation,
		version: &Version,
		old_version: &Option<Version>,
	) -> HistoryPackage {
		Self {
			name: version.parent().name().to_string(),
			version: version.version().to_string(),
			old_version: old_version.as_ref().map(|ver| ver.version().to_string()),
			size: version.size(),
			operation,
			auto_installed: version.parent().is_auto_installed(),
			items: std::cell::OnceCell::new(),
			changelog: OnceCell::new(),
		}
	}

	pub fn get_pkg<'a>(&self, cache: &'a Cache) -> Result<Package<'a>> {
		if let Some(pkg) = cache.get(&self.name) {
			return Ok(pkg);
		}
		bail!("Package '{}' not found in cache", self.name)
	}

	pub fn get_version<'a>(&self, cache: &'a Cache) -> Result<Version<'a>> {
		if let Some(ver) = self.get_pkg(cache)?.get_version(&self.version) {
			return Ok(ver);
		}
		bail!("Version '{}' not found for '{}'", self.version, self.name)
	}

	pub async fn get_changelog(&self, cache: &Cache) -> Result<&String> {
		self.changelog
			.get_or_try_init(|| async {
				let uri = match self.get_pkg(cache)?.changelog_uri() {
					Some(uri) => uri,
					None => bail!("Unable to find Changelog URI"),
				};

				Ok(reqwest::get(uri).await?.error_for_status()?.text().await?)
			})
			.await
	}

	pub fn items(&self, config: &Config) -> &Vec<String> {
		self.items.get_or_init(|| {
			let colored = color::color!(self.operation.into(), &self.name).to_string();
			let mut items = vec![colored];

			if let Some(old) = &self.old_version {
				items.push(old.to_string());
				items.push(version_diff(old, &self.version));
			} else {
				items.push(self.version.to_string());
			}

			items.push(config.unit_str(self.size));
			items
		})
	}

	pub async fn render_changelog(&self, cache: &Cache, terminal: &mut tui::Term) -> Result<()> {
		let changelog = match self.get_changelog(cache).await {
			Ok(log) => log,
			Err(e) => &format!("{e:?}"),
		};

		let mut pager = std::process::Command::new("less")
			.arg("--raw-control-chars")
			.arg("--clear-screen")
			.stdin(std::process::Stdio::piped())
			.spawn()?;

		if let Some(stdin) = pager.stdin.as_mut() {
			if let Err(err) = stdin.write_all(changelog.as_bytes()) {
				match err.kind() {
					// Broken Pipe if not all of the changelog is read.
					// Happens on pager exit without reading the whole file.
					std::io::ErrorKind::BrokenPipe => {},
					_ => return Err(err.into()),
				}
			}
		}

		pager.wait()?;
		execute!(
			terminal.backend_mut(),
			EnterAlternateScreen,
			EnableMouseCapture
		)?;
		terminal.clear()?;

		Ok(())
	}

	pub fn render_show(
		&self,
		cache: &Cache,
		config: &Config,
		terminal: &mut tui::Term,
	) -> Result<()> {
		// Maybe we will show both versions if available?
		let show = ShowVersion::new(self.get_version(cache)?);
		terminal.clear()?;

		let mut lines: Vec<Text> = vec![];
		for (head, info) in show.pretty_map() {
			let mut split = info.split('\n');
			if let Some(first) = split.next() {
				lines.push(format!("{}: {first}", color::highlight!(head)).into_text()?);
				for line in split {
					let line = line.to_string();
					lines.push(line.into_text()?)
				}
			}
		}

		loop {
			terminal.term.draw(|f| {
				let block = tui::summary::header_block(config, "Nala Upgrade");

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

pub async fn get_history(config: &Config) -> Result<Vec<HistoryEntry>> {
	let history_db = config.get_path(&Paths::History);
	if !history_db.exists() {
		history_db.mkdir().await?;
	}

	let mut history = std::fs::read_dir(&history_db)
		.with_context(|| format!("{}", history_db.display()))?
		.filter_map(|dir_entry| {
			let path = dir_entry.ok()?.path();

			if !path.is_file() {
				return None;
			}

			let filename = path.file_name()?.to_str()?;
			crate::debug!("File '{filename}' found");
			let id = match filename.split('.').next()?.parse::<u64>() {
				Ok(num) => num,
				Err(e) => {
					crate::error!("{:?}", anyhow!(e).context("Filename is not an int."));
					return None;
				},
			};

			Some((id, path))
		})
		.collect::<Vec<_>>();

	history.sort_by_cached_key(|p| p.0);

	let mut parsed = vec![];

	for (_, path) in history {
		parsed.push(
			serde_json::from_slice::<HistoryEntry>(path.read_string().await?.as_bytes())
				// bincode::deserialize::<HistoryEntry>(path.read_string().await?.as_bytes())
				.with_context(|| format!("Unable to deserialize '{}'", path.display()))?,
		);
	}

	Ok(parsed)
}

pub fn version_diff(old: &str, new: &str) -> String {
	// Check for just revision change first.
	if let (Some(old_ver), Some(new_ver)) = (old.rsplit_once('-'), new.rsplit_once('-')) {
		// If there isn't a revision these shouldn't ever match
		// If they do match then only the revision has changed
		if old_ver.0 == new_ver.0 {
			return format!("{}-{}", new_ver.0, color::color!(Theme::Notice, new_ver.0));
		}
	}

	let (old_ver, new_ver) = (
		old.split('.').collect::<Vec<_>>(),
		new.split('.').collect::<Vec<_>>(),
	);

	let mut start_color = 0;
	for (i, section) in old_ver.iter().enumerate() {
		if i > new_ver.len() - 1 {
			break;
		}

		if section != &new_ver[i] {
			start_color = i;
			break;
		}
	}

	new_ver
		.iter()
		.enumerate()
		.map(|(i, str)| {
			if i >= start_color {
				color::color!(Theme::Notice, str).to_string()
			} else {
				str.to_string()
			}
		})
		.collect::<Vec<_>>()
		.join(".")
}
