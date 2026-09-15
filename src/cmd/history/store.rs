use std::path::Path;

use anyhow::{bail, Context, Result};
use nix::fcntl::{renameat2, RenameFlags, AT_FDCWD};

use super::legacy::{legacy_history_path, read_legacy_history};
use super::model::{HistoryEntry, HISTORY_SCHEMA_VERSION};
use crate::cli::HistorySelector;
use crate::config::{Config, Paths};
use crate::t;
use crate::{debug, warn};

const LEGACY_HISTORY_MARKER: &str = ".legacy-history-handled";

/// Removes temporary history state left by an interrupted Nala mutation.
pub(crate) fn cleanup_stale_history(config: &Config) -> Result<()> {
	let history_dir = config.get_path(&Paths::History);
	if history_dir.exists() {
		for entry in std::fs::read_dir(&history_dir)
			.with_context(|| t!("file-read", "path" => history_dir.display().to_string()))?
		{
			let path = entry?.path();
			let Some(stem) = path
				.file_name()
				.and_then(|name| name.to_str())
				.and_then(|name| name.strip_suffix(".json.tmp"))
			else {
				continue;
			};
			if path.is_file() && stem.parse::<u32>().is_ok() {
				std::fs::remove_file(&path)
					.with_context(|| t!("file-remove", "path" => path.display().to_string()))?;
			}
		}
	}

	let Some(parent) = history_dir.parent() else {
		return Ok(());
	};
	if !parent.exists() {
		return Ok(());
	}
	let name = history_dir
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("history");
	let prefix = format!(".{name}.importing-");
	for entry in std::fs::read_dir(parent)
		.with_context(|| t!("file-read", "path" => parent.display().to_string()))?
	{
		let path = entry?.path();
		if path.is_dir()
			&& path
				.file_name()
				.and_then(|name| name.to_str())
				.is_some_and(|name| name.starts_with(&prefix))
		{
			std::fs::remove_dir_all(&path)
				.with_context(|| t!("file-remove", "path" => path.display().to_string()))?;
		}
	}

	Ok(())
}

fn history_entry_id(path: &Path) -> Option<u32> {
	if path.extension()? != "json" {
		return None;
	}

	path.file_stem()?.to_str()?.parse().ok()
}

/// Reads and deserializes every stored history entry from the history directory.
pub fn get_history(config: &Config) -> Result<Vec<HistoryEntry>> {
	let history_db = config.get_path(&Paths::History);
	let mut current = if history_db.exists() {
		read_history_dir(&history_db)?
	} else {
		Vec::new()
	};

	let legacy_path = legacy_history_path(&history_db);
	if !legacy_path.exists() || history_db.join(LEGACY_HISTORY_MARKER).exists() {
		return Ok(current);
	}

	let mut history = read_legacy_history(&legacy_path)?;
	history.append(&mut current);
	for (id, entry) in (1_u32..).zip(&mut history) {
		entry.id = id;
	}
	Ok(history)
}

fn read_history_dir(history_db: &Path) -> Result<Vec<HistoryEntry>> {
	let mut history = vec![];
	for dir_entry in
		std::fs::read_dir(history_db)
			.with_context(|| t!("file-read", "path" => history_db.display().to_string()))?
	{
		let path = dir_entry?.path();
		if !path.is_file() {
			continue;
		}

		let Some(filename_id) = history_entry_id(&path) else {
			debug!("Skipping non-history file '{}'", path.display());
			continue;
		};

		debug!("File '{}' found", path.display());
		history.push(read_history_entry(&path, filename_id)?);
	}

	history.sort_by_key(|entry| entry.id);
	Ok(history)
}

fn read_history_entry(path: &Path, filename_id: u32) -> Result<HistoryEntry> {
	let raw = std::fs::read(path)
		.with_context(|| t!("file-read", "path" => path.display().to_string()))?;
	let value = serde_json::from_slice::<serde_json::Value>(&raw)
		.with_context(|| t!("file-deserialize", "path" => path.display().to_string()))?;
	let Some(schema_version) = value.get("schema_version").and_then(|value| value.as_u64()) else {
		bail!(
			"History entry '{}' has no valid schema_version; expected {HISTORY_SCHEMA_VERSION}",
			path.display()
		);
	};

	if schema_version != u64::from(HISTORY_SCHEMA_VERSION) {
		bail!(
			"History entry '{}' uses unsupported schema version {schema_version}; supported version is {HISTORY_SCHEMA_VERSION}. Use a compatible Nala version or move this file aside",
			path.display()
		);
	}

	let entry = serde_json::from_value::<HistoryEntry>(value)
		.with_context(|| t!("file-deserialize", "path" => path.display().to_string()))?;
	if entry.id != filename_id {
		bail!(
			"History entry '{}' contains ID {}; expected {filename_id} from its filename",
			path.display(),
			entry.id
		);
	}

	Ok(entry)
}

/// Returns the next transaction ID for the on-disk history store.
pub(super) fn next_history_id(config: &Config) -> Result<u32> {
	Ok(get_history(config)?
		.iter()
		.map(|entry| entry.id)
		.max()
		.unwrap_or_default()
		+ 1)
}

/// Promotes legacy history and validates the store before package mutation.
pub(crate) fn prepare_history_store(config: &Config) -> Result<u32> {
	migrate_legacy_history(config)?;
	next_history_id(config)
}

/// Clears a stored history entry by durable selector, or removes all entries.
pub fn clear_history(
	config: &Config,
	entries: &[HistoryEntry],
	selector: Option<&HistorySelector>,
	clear_all: bool,
) -> Result<usize> {
	let history_dir = config.get_path(&Paths::History);
	migrate_legacy_history(config)?;

	if clear_all {
		if !history_dir.exists() {
			return Ok(0);
		}

		let mut removed = 0;
		for dir_entry in
			std::fs::read_dir(&history_dir)
				.with_context(|| t!("file-read", "path" => history_dir.display().to_string()))?
		{
			let path = dir_entry?.path();
			if !path.is_file() {
				continue;
			}
			if history_entry_id(&path).is_none() {
				continue;
			}

			std::fs::remove_file(&path)
				.with_context(|| t!("file-remove", "path" => path.display().to_string()))?;
			removed += 1;
		}

		return Ok(removed);
	}

	let Some(selector) = selector else {
		bail!("{}", t!("history-clear-target"));
	};

	let entry = HistoryEntry::find_selector(entries, selector)?;
	let filename = history_dir.join(format!("{}.json", entry.id));
	std::fs::remove_file(&filename)
		.with_context(|| t!("file-remove", "path" => filename.display().to_string()))?;
	Ok(1)
}

impl HistoryEntry {
	/// Serializes this entry into the per-transaction history store.
	pub fn write_to_file(&self, config: &Config) -> Result<()> {
		let history_dir = config.get_path(&Paths::History);
		migrate_legacy_history(config)?;
		std::fs::create_dir_all(&history_dir)
			.with_context(|| t!("file-create", "path" => history_dir.display().to_string()))?;

		self.write_to_dir(&history_dir)
	}

	fn write_to_dir(&self, history_dir: &Path) -> Result<()> {
		let filename = history_dir.join(format!("{}.json", self.id));
		let tmp_filename = filename.with_extension("json.tmp");

		let mut serialized =
			serde_json::to_vec_pretty(self).context(t!("history-serialize"))?;
		serialized.push(b'\n');

		std::fs::write(&tmp_filename, serialized)
			.with_context(|| {
				t!("file-write", "path" => tmp_filename.display().to_string())
			})?;
		std::fs::rename(&tmp_filename, &filename)
			.with_context(|| {
				t!("file-replace", "path" => filename.display().to_string())
			})?;

		Ok(())
	}
}

fn migrate_legacy_history(config: &Config) -> Result<()> {
	cleanup_stale_history(config)?;
	let history_dir = config.get_path(&Paths::History);
	let legacy_path = legacy_history_path(&history_dir);
	if !legacy_path.exists() || history_dir.join(LEGACY_HISTORY_MARKER).exists() {
		return Ok(());
	}

	let entries = get_history(config)?;
	let name = history_dir
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or("history");
	let staging_dir =
		history_dir.with_file_name(format!(".{name}.importing-{}", std::process::id()));
	std::fs::create_dir(&staging_dir)
		.with_context(|| t!("file-create", "path" => staging_dir.display().to_string()))?;

	let migration = (|| -> Result<()> {
		for entry in &entries {
			entry.write_to_dir(&staging_dir)?;
		}
		let marker = staging_dir.join(LEGACY_HISTORY_MARKER);
		std::fs::write(&marker, b"")
			.with_context(|| t!("file-write", "path" => marker.display().to_string()))?;

		if history_dir.exists() {
			renameat2(
				AT_FDCWD,
				&staging_dir,
				AT_FDCWD,
				&history_dir,
				RenameFlags::RENAME_EXCHANGE,
			)
			.with_context(|| t!("file-replace", "path" => history_dir.display().to_string()))?;
			if let Err(error) = std::fs::remove_dir_all(&staging_dir) {
				warn!(
					"History migration succeeded, but the old store at '{}' could not be removed: {error}",
					staging_dir.display()
				);
			}
		} else {
			std::fs::rename(&staging_dir, &history_dir)
				.with_context(|| t!("file-replace", "path" => history_dir.display().to_string()))?;
		}

		Ok(())
	})();

	if let Err(error) = migration {
		if let Err(cleanup_error) = std::fs::remove_dir_all(&staging_dir)
			&& cleanup_error.kind() != std::io::ErrorKind::NotFound
		{
			warn!(
				"Failed to clean incomplete history migration at '{}': {cleanup_error}",
				staging_dir.display()
			);
		}
		return Err(error);
	}

	debug!(
		"Migrated combined history store with {} entries after importing '{}'",
		entries.len(),
		legacy_path.display()
	);
	Ok(())
}
