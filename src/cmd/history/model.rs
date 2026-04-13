use serde::{Deserialize, Serialize};

use crate::cli::HistorySelector;
use crate::config::Config;
use crate::libnala::PackageTransition;
use crate::util;

/// Schema version for on-disk package transaction history entries.
pub const HISTORY_SCHEMA_VERSION: u32 = 1;

/// Outcome state recorded for a stored history entry.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum HistoryStatus {
	#[default]
	Unknown,
	Applied,
}

/// Stored package transaction record written by the history command path.
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
	pub schema_version: u32,
	pub id: u32,
	pub started_at: String,
	pub finished_at: String,
	pub status: HistoryStatus,
	pub requested_by: String,
	pub command: String,
	pub requested_targets: Vec<String>,
	pub altered: usize,
	pub(super) packages: Vec<PackageTransition>,
}

impl HistoryEntry {
	/// Builds a successful package transaction entry using the current CLI context.
	pub fn applied(
		config: &Config,
		id: u32,
		started_at: String,
		finished_at: String,
		packages: Vec<PackageTransition>,
	) -> Self {
		let (uid, username) = util::get_user();
		Self {
			schema_version: HISTORY_SCHEMA_VERSION,
			id,
			started_at,
			finished_at,
			status: HistoryStatus::Applied,
			requested_by: format!("{username} ({uid})"),
			command: std::env::args().skip(1).collect::<Vec<String>>().join(" "),
			requested_targets: config
				.get_vec(crate::config::keys::PKG_NAMES)
				.cloned()
				.unwrap_or_default(),
			altered: packages.len(),
			packages,
		}
	}

	/// Returns the package effects recorded for this transaction.
	pub fn packages(&self) -> &[PackageTransition] {
		&self.packages
	}

	/// Selects a stored history entry by durable transaction ID or the latest
	/// available record.
	pub fn find_selector<'a>(
		entries: &'a [Self],
		selector: &HistorySelector,
	) -> anyhow::Result<&'a Self> {
		match selector {
			HistorySelector::Last => entries
				.iter()
				.max_by_key(|entry| entry.id)
				.ok_or_else(|| anyhow::anyhow!("No history entries found.")),
			HistorySelector::Id(id) => Self::find(entries, *id),
		}
	}
}
