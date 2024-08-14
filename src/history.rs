use std::collections::HashMap;
use std::io::Write;
use std::{fs, io};

use ansi_to_tui::IntoText;
use anyhow::{anyhow, bail, Context, Result};
use chrono::{DateTime, Local, Utc};
use crossterm::event::{self, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use ratatui::layout::Constraint::Length;
use ratatui::layout::Layout;
use ratatui::text::Text;
use ratatui::widgets::{Paragraph, Wrap};
use rust_apt::{new_cache, Cache, Package, Version};
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::colors::Theme;
use crate::config::{Config, Paths};
use crate::show::{build_regex, show_version};
use crate::{dprint, table, tui, util};

#[derive(Serialize, Deserialize)]
pub struct HistoryFile {
	entries: Vec<HistoryEntry>,
	version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
	pub id: u32,
	pub date: String,
	pub requested_by: String,
	pub command: String,
	pkg_names: Vec<String>,
	pub altered: usize,
	packages: Vec<HistoryPackage>,
}

impl HistoryEntry {
	pub fn new(id: u32, date: String, packages: Vec<HistoryPackage>) -> Self {
		let (uid, username) = util::get_user();
		Self {
			id,
			date,
			requested_by: format!("{username} ({uid})"),
			command: std::env::args().skip(1).collect::<Vec<String>>().join(" "),
			pkg_names: vec!["I don't know if we need this".to_string()],
			altered: packages.len(),
			packages,
		}
	}

	pub fn write_to_file(&self, config: &Config) -> Result<()> {
		let mut filename = config.get_path(&Paths::History);
		filename.push(format!("{}.bin", self.id));

		fs::write(
			&filename,
			bincode::serialize(&self)
				.with_context(|| format!("Unable to serialize HistoryEntry\n\n    {self:?}"))?,
		)
		.with_context(|| format!("Unable to write to '{}'", filename.display()))?;

		Ok(())
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HistoryPackage {
	pub name: String,
	pub version: String,
	pub old_version: Option<String>,
	pub size: u64,
	pub operation: Operation,
	pub auto_installed: bool,
	#[serde(skip)]
	items: std::cell::OnceCell<Vec<tui::summary::Item>>,
	#[serde(skip)]
	changelog: OnceCell<String>,
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

	pub fn items(&self, config: &Config) -> &Vec<tui::summary::Item> {
		self.items.get_or_init(|| {
			let secondary = config.rat_style(self.operation.theme());
			let primary = config.rat_style(Theme::Regular);

			let colored = config.color(self.operation.theme(), &self.name);
			let mut items = vec![tui::summary::Item::left(secondary, colored)];

			if let Some(old) = &self.old_version {
				items.push(tui::summary::Item::center(primary, old.to_string()));
				items.push(tui::summary::Item::center(
					primary,
					util::version_diff(config, old, self.version.to_string()),
				));
			} else {
				items.push(tui::summary::Item::center(
					primary,
					self.version.to_string(),
				));
			}
			items.push(tui::summary::Item::right(
				primary,
				config.unit_str(self.size),
			));
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
					io::ErrorKind::BrokenPipe => {},
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
		let pkg = self.get_pkg(cache)?;
		let pacstall_regex = build_regex(r#"_remoterepo="(.*?)""#)?;
		let url_regex = build_regex("(https?://.*?/.*?/)")?;
		// Maybe we will show both versions if available?
		let show = show_version(
			config,
			&pkg,
			&self.get_version(cache)?,
			&pacstall_regex,
			&url_regex,
		);
		terminal.clear()?;

		let mut lines: Vec<Text> = vec![];
		for (head, info) in &show {
			let mut split = info.split('\n');
			if let Some(first) = split.next() {
				lines.push(
					format!("{}: {first}", config.color(Theme::Highlight, head)).into_text()?,
				);
				for line in split {
					let line = line.to_string();
					lines.push(line.into_text()?)
				}
			}
		}

		loop {
			terminal.draw(|f| {
				let block = tui::summary::header_block(config, "Nala Upgrade");

				let inner = block.inner(f.size());

				let constraints = lines
					.iter()
					.map(|line| Length((line.width() as f32 / inner.width as f32).ceil() as u16))
					.collect::<Vec<_>>();

				let layout = Layout::vertical(constraints).split(block.inner(f.size()));

				f.render_widget(block, f.size());
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Operation {
	Remove,
	AutoRemove,
	Purge,
	AutoPurge,
	Install,
	Reinstall,
	Upgrade,
	Downgrade,
	Held,
}

impl Operation {
	pub fn to_vec() -> Vec<Operation> {
		vec![
			Self::Remove,
			Self::AutoRemove,
			Self::Purge,
			Self::AutoPurge,
			Self::Install,
			Self::Reinstall,
			Self::Upgrade,
			Self::Downgrade,
		]
	}

	pub fn as_str(&self) -> &'static str {
		match self {
			Operation::Remove => "Remove",
			Operation::AutoRemove => "AutoRemove",
			Operation::Purge => "Purge",
			Operation::AutoPurge => "AutoPurge",
			Operation::Install => "Install",
			Operation::Reinstall => "ReInstall",
			Operation::Upgrade => "Upgrade",
			Operation::Downgrade => "Downgrade",
			Operation::Held => "Held",
		}
	}

	pub fn theme(&self) -> Theme {
		match self {
			Self::Remove | Self::AutoRemove | Self::Purge | Self::AutoPurge => Theme::Error,
			Self::Install | Self::Upgrade => Theme::Secondary,
			Self::Reinstall | Self::Downgrade | Self::Held => Theme::Notice,
		}
	}
}

impl std::fmt::Display for Operation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

pub fn get_history(config: &Config) -> Result<Vec<HistoryEntry>> {
	let history_db = config.get_path(&Paths::History);
	if !history_db.exists() {
		std::fs::create_dir_all(&history_db)
			.with_context(|| format!("Could not create {}", history_db.display()))?;
	}

	let mut history = std::fs::read_dir(&history_db)
		.with_context(|| format!("{}", history_db.display()))?
		.filter_map(|dir_entry| {
			let path = dir_entry.ok()?.path();

			if !path.is_file() {
				return None;
			}

			let filename = path.file_name()?.to_str()?;
			dprint!(config, "File '{filename}' found");
			let id = match filename.split('.').next()?.parse::<u64>() {
				Ok(num) => num,
				Err(e) => {
					config.stderr(
						Theme::Error,
						&format!("{:?}", anyhow!(e).context("Filename is not an int.")),
					);
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
			// serde_json::from_slice::<HistoryEntry>(
			bincode::deserialize::<HistoryEntry>(
				&std::fs::read(&path)
					.with_context(|| format!("Unable to read '{}'", path.display()))?,
			)
			.with_context(|| format!("Unable to deserialize '{}'", path.display()))?,
		);
	}

	Ok(parsed)
}

#[tokio::main]
pub async fn history(config: &Config) -> Result<()> {
	let history_file = get_history(config)?;
	let cache = new_cache!()?;

	let mut table = table::get_table(
		config,
		&["ID", "Command", "Date and Time", "Requested-By", "Altered"],
	);

	// TODO: Make it configurable which timezones you want.

	// Convert Stored UTC into the local time zone
	let date_times = history_file
		.iter()
		.filter_map(|e| {
			Some(
				e.date
					.parse::<DateTime<Utc>>()
					.ok()?
					.with_timezone(&Local)
					.format("%Y-%m-%d %H:%M:%S %Z"),
			)
		})
		.collect::<Vec<_>>();

	for (i, entry) in history_file.iter().enumerate() {
		let row: Vec<&dyn std::fmt::Display> = vec![
			&entry.id,
			&entry.command,
			&date_times[i],
			&entry.requested_by,
			&entry.altered,
		];
		table.add_row(row);
	}

	if !config.show_tui() {
		println!("{table}");
		return Ok(());
	}

	// This will actually be the `history info` command.
	let mut pkg_set: HashMap<Operation, Vec<HistoryPackage>> = HashMap::new();

	let num = 2;

	let Some(entry) = history_file.into_iter().nth(num - 1) else {
		bail!("History entry with ID '{num}' does not exist")
	};

	for pkg in entry.packages {
		pkg_set.entry(pkg.operation).or_default().push(pkg)
	}

	tui::summary::SummaryTab::new(&cache, config, &pkg_set)
		.run()
		.await?;

	Ok(())
}
