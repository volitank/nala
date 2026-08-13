use std::ffi::OsString;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;
use serde_json::Value;

use super::model::{HistoryEntry, HistoryStatus, HISTORY_SCHEMA_VERSION};
use crate::libnala::{Operation, PackageState, PackageTransition};
use crate::t;

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
struct LegacyHistoryEntry {
	date: String,
	#[serde(rename = "Requested-By")]
	requested_by: String,
	command: Vec<String>,
	purged: bool,
	explicit: Vec<String>,
	removed: Vec<Vec<Value>>,
	#[serde(rename = "Auto-Removed")]
	auto_removed: Vec<Vec<Value>>,
	installed: Vec<Vec<Value>>,
	reinstalled: Vec<Vec<Value>>,
	upgraded: Vec<Vec<Value>>,
	downgraded: Vec<Vec<Value>>,
}

struct LegacyPackage {
	name: String,
	version: Option<String>,
	old_version: Option<String>,
	size: u64,
}

pub(super) fn legacy_history_path(history_dir: &Path) -> PathBuf {
	let mut path = OsString::from(history_dir.as_os_str());
	path.push(".json");
	PathBuf::from(path)
}

pub(super) fn read_legacy_history(path: &Path) -> Result<Vec<HistoryEntry>> {
	let raw = std::fs::read(path)
		.with_context(|| t!("file-read", "path" => path.display().to_string()))?;
	let history = serde_json::from_slice::<serde_json::Map<String, Value>>(&raw)
		.with_context(|| t!("file-deserialize", "path" => path.display().to_string()))?;
	let mut entries = Vec::new();

	for (key, value) in history {
		if key == "Nala" {
			continue;
		}

		let id = key
			.parse::<u32>()
			.with_context(|| format!("Invalid legacy history ID '{key}'"))?;
		let entry = serde_json::from_value::<LegacyHistoryEntry>(value)
			.with_context(|| format!("Invalid legacy history entry '{id}'"))?;
		entries.push(entry.convert(id).with_context(|| {
			format!("Failed to convert legacy history entry '{id}' from '{}'", path.display())
		})?);
	}

	entries.sort_by_key(|entry| entry.id);
	Ok(entries)
}

impl LegacyHistoryEntry {
	fn convert(self, id: u32) -> Result<HistoryEntry> {
		let purged = self.purged
			|| self
				.command
				.first()
				.is_some_and(|command| matches!(command.as_str(), "purge" | "autopurge"));
		let mut packages = Vec::new();

		packages.extend(convert_packages(
			self.removed,
			if purged { Operation::Purge } else { Operation::Remove },
		)?);
		packages.extend(convert_packages(
			self.auto_removed,
			if purged { Operation::AutoPurge } else { Operation::AutoRemove },
		)?);
		packages.extend(convert_packages(self.installed, Operation::Install)?);
		packages.extend(convert_packages(self.reinstalled, Operation::Reinstall)?);
		packages.extend(convert_packages(self.upgraded, Operation::Upgrade)?);
		packages.extend(convert_packages(self.downgraded, Operation::Downgrade)?);

		Ok(HistoryEntry {
			schema_version: HISTORY_SCHEMA_VERSION,
			id,
			started_at: self.date.clone(),
			finished_at: self.date,
			status: HistoryStatus::Applied,
			requested_by: if self.requested_by.is_empty() {
				t!("unknown")
			} else {
				self.requested_by
			},
			command: self.command.join(" "),
			requested_targets: self.explicit,
			packages,
		})
	}
}

fn convert_packages(rows: Vec<Vec<Value>>, operation: Operation) -> Result<Vec<PackageTransition>> {
	rows.into_iter()
		.map(|row| {
			let package = LegacyPackage::parse(&row, operation)?;
			let installed = |version| PackageState { version, ..Default::default() };

			let (before, after) = match operation {
				Operation::Install => (
					PackageState::config_only(None, None),
					installed(package.version.clone()),
				),
				Operation::Reinstall => {
					let state = installed(package.version.clone());
					(state.clone(), state)
				},
				Operation::Upgrade | Operation::Downgrade => (
					installed(package.old_version),
					installed(package.version.clone()),
				),
				Operation::Remove | Operation::AutoRemove => (
					installed(package.version.clone()),
					PackageState::config_only(package.version.clone(), None),
				),
				Operation::Purge | Operation::AutoPurge => {
					(installed(package.version.clone()), PackageState::missing())
				},
				Operation::Configure | Operation::Held => unreachable!(),
			};

			Ok(PackageTransition::transition(
				package.name,
				package.size,
				operation,
				before,
				after,
			))
		})
		.collect()
}

impl LegacyPackage {
	fn parse(row: &[Value], operation: Operation) -> Result<Self> {
		let Some((name, fields)) = row.split_first() else {
			bail!("Legacy history package row is empty")
		};
		let name = value_string(name)?;
		let (version, old_version, size) = match fields {
			[version, size] => (version, None, value_size(size)?),
			[_, _, _] if !matches!(operation, Operation::Upgrade | Operation::Downgrade) => {
				bail!("Invalid four-field legacy history package row for '{name}'")
			},
			[first, second, third] => match value_size(second) {
				// Match the Python reader: modern layout wins when both sizes are numeric.
				Ok(size) => (first, Some(third), size),
				Err(_) => (second, Some(first), value_size(third)?),
			},
			_ => bail!("Invalid legacy history package row for '{name}'"),
		};

		Ok(Self {
			name,
			version: optional_version(version)?,
			old_version: old_version.map(optional_version).transpose()?.flatten(),
			size,
		})
	}
}

fn value_string(value: &Value) -> Result<String> {
	match value {
		Value::String(value) => Ok(value.clone()),
		Value::Number(value) => Ok(value.to_string()),
		_ => bail!("Invalid legacy history value '{value}'"),
	}
}

fn optional_version(value: &Value) -> Result<Option<String>> {
	if value.is_null() {
		return Ok(None);
	}

	let version = value_string(value)?;
	Ok((!version.is_empty() && version != "None").then_some(version))
}

fn value_size(value: &Value) -> Result<u64> {
	value_string(value)?
		.parse()
		.with_context(|| format!("Invalid legacy history package size '{value}'"))
}
