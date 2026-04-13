use std::fs;

use anyhow::{Context, Result};

use super::model::HistoryEntry;
use crate::config::{Config, Paths};
use crate::{debug, warn};
use crate::fs::AsyncFs;

const TMP_EXTENSION: &str = "tmp";

/// Reads and deserializes every stored history entry from the history directory.
pub async fn get_history(config: &Config) -> Result<Vec<HistoryEntry>> {
	let history_db = config.get_path(&Paths::History);
	if !history_db.exists() {
		history_db.mkdir().await?;
	}

	let mut history = vec![];
	for dir_entry in std::fs::read_dir(&history_db)
		.with_context(|| format!("{}", history_db.display()))?
	{
		let path = dir_entry?.path();
		if !path.is_file() {
			continue;
		}

		if path.extension().is_some_and(|ext| ext == TMP_EXTENSION) {
			warn!("Skipping temp history file '{}'", path.display());
			continue;
		}

		debug!("File '{}' found", path.display());
		history.push(
			serde_json::from_slice::<HistoryEntry>(
				&std::fs::read(&path)
					.with_context(|| format!("Unable to read '{}'", path.display()))?,
			)
			.with_context(|| format!("Unable to deserialize '{}'", path.display()))?,
		);
	}

	history.sort_by_key(|entry| entry.id);
	Ok(history)
}

/// Returns the next transaction ID for the on-disk history store.
pub async fn next_history_id(config: &Config) -> Result<u32> {
	Ok(get_history(config)
		.await?
		.iter()
		.map(|entry| entry.id)
		.max()
		.unwrap_or_default()
		+ 1)
}

impl HistoryEntry {
	/// Serializes this entry into the per-transaction history store.
	pub fn write_to_file(&self, config: &Config) -> Result<()> {
		let history_dir = config.get_path(&Paths::History);
		if !history_dir.exists() {
			fs::create_dir_all(&history_dir)
				.with_context(|| format!("Unable to create '{}'", history_dir.display()))?;
		}

		let mut filename = history_dir.clone();
		filename.push(format!("{}.json", self.id));
		let tmp_filename = filename.with_extension("json.tmp");

		let mut serialized = serde_json::to_vec_pretty(self)
			.with_context(|| format!("Unable to serialize HistoryEntry\n\n    {self:?}"))?;
		serialized.push(b'\n');

		fs::write(&tmp_filename, serialized)
			.with_context(|| format!("Unable to write to '{}'", tmp_filename.display()))?;
		fs::rename(&tmp_filename, &filename)
			.with_context(|| format!("Unable to replace '{}'", filename.display()))?;

		Ok(())
	}
}
