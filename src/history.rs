use anyhow::Result;
use chrono::Utc;
use rust_apt::new_cache;
use rust_apt::package::Version;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::util::{get_user, geteuid};

#[derive(Serialize, Deserialize)]
pub struct HistoryFile {
	entries: Vec<HistoryEntry>,
	version: String,
}

#[derive(Serialize, Deserialize)]
pub struct HistoryEntry {
	id: u32,
	date: String,
	requested_by: String,
	command: String,
	pkg_names: Vec<String>,
	altered: u32,
	purged: bool,
	operation: Operation,
	packages: Vec<HistoryPackage>,
}

#[derive(Serialize, Deserialize)]
pub struct HistoryPackage {
	name: String,
	version: String,
	old_version: Option<String>,
	size: u64,
	operation: Operation,
	auto_installed: bool,
}

impl HistoryPackage {
	fn from_version(version: Version, old_version: Option<Version>) -> HistoryPackage {
		HistoryPackage {
			name: version.parent().name().to_string(),
			version: version.version().to_string(),
			old_version: old_version.map(|ver| ver.version().to_string()),
			size: version.size(),
			// For Now Hard Code the operation?
			operation: Operation::Install,
			auto_installed: version.parent().is_auto_installed(),
		}
	}
}

#[derive(Serialize, Deserialize)]
enum Operation {
	Remove,
	AutoRemove,
	Purge,
	AutoPurge,
	Install,
	Reinstall,
	Upgrade,
	Downgrade,
}

pub fn history_test(config: &Config) -> Result<()> {
	let cache = new_cache!().unwrap();

	if let Some(pkg_names) = config.pkg_names() {
		let date = Utc::now().to_rfc3339();

		let mut packages = vec![];
		for pkg_name in pkg_names {
			let pkg = cache.get(pkg_name).unwrap();

			packages.push(HistoryPackage::from_version(
				pkg.candidate().unwrap(),
				pkg.installed(),
			))
		}

		let entry = HistoryEntry {
			id: 1,
			date,
			requested_by: format!("{} ({})", get_user(), unsafe { geteuid().to_string() }),
			command: std::env::args().skip(1).collect::<Vec<String>>().join(" "),
			pkg_names: pkg_names.to_vec(),
			altered: 12,
			purged: false,
			operation: Operation::Install,
			packages,
		};

		let history = HistoryFile {
			entries: vec![entry],
			version: "0.2.0".to_string(),
		};

		let json = serde_json::to_string_pretty(&history)?;
		println!("{json}");
	}
	Ok(())
}
