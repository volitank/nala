macro_rules! define_modules {
	($($module:ident),*) => {
		$(
			mod $module;
			pub use $module::$module;
		)*
	};
}

define_modules!(show, update, upgrade, history, fetch, clean);

pub mod install;
mod list;
pub mod traits;

use anyhow::Result;
// TODO: These should maybe be part of like a libnala?
pub use history::{get_history, HistoryEntry, HistoryPackage};
use indexmap::IndexMap;
pub use install::mark_cli_pkgs;
pub use list::list_packages;
use rust_apt::records::RecordField;
use rust_apt::{DepType, Version};
use serde::{Deserialize, Serialize};
use show::format_local;
use traits::ShowFormat;
pub use upgrade::{apt_hook_with_pkgs, ask, run_scripts};

use crate::config::{color, Config, Theme};
use crate::util::URL;

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
}

impl Operation {
	pub fn as_str(&self) -> &str { self.as_ref() }
}

impl std::fmt::Display for Operation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", AsRef::<str>::as_ref(self))
	}
}

impl AsRef<str> for Operation {
	fn as_ref(&self) -> &str {
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
}

impl AsRef<Theme> for Operation {
	fn as_ref(&self) -> &Theme {
		match self {
			Self::Remove | Self::AutoRemove | Self::Purge | Self::AutoPurge => &Theme::Error,
			Self::Install | Self::Upgrade => &Theme::Secondary,
			Self::Reinstall | Self::Downgrade | Self::Held => &Theme::Notice,
		}
	}
}

const DEP_ITER: &[DepType] = {
	&[
		DepType::Depends,
		DepType::PreDepends,
		DepType::Suggests,
		DepType::Recommends,
		DepType::Conflicts,
		DepType::Replaces,
		DepType::Obsoletes,
		DepType::DpkgBreaks,
		DepType::Enhances,
	]
};

const RECORDS: [&str; 13] = [
	RecordField::Package,
	RecordField::Version,
	RecordField::Architecture,
	RecordField::Priority,
	RecordField::Essential,
	RecordField::Section,
	RecordField::Source,
	RecordField::InstalledSize,
	RecordField::Size,
	RecordField::Maintainer,
	RecordField::OriginalMaintainer,
	RecordField::Homepage,
	RecordField::SHA256,
];

fn print_info(header: &str, value: &str) {
	let sep = color::highlight!(":");
	let header = color::highlight!(header);
	println!("{header}{sep} {value}")
}

struct ShowVersion<'a> {
	ver: Version<'a>,
	records: IndexMap<&'static str, String>,
}

impl ShowVersion<'_> {
	pub fn new(ver: Version) -> ShowVersion {
		let records = IndexMap::from_iter(RECORDS.iter().copied().map(|key| {
			(
				key,
				ver.get_record(key).unwrap_or_else(|| "Unknown".to_string()),
			)
		}));
		ShowVersion { ver, records }
	}

	pub fn map(&self) -> IndexMap<&str, String> {
		let mut map = IndexMap::new();

		for (key, value) in &self.records {
			map.insert(*key, value.to_string());
		}

		// Package File Section
		if let Some(pkg_file) = self.ver.package_files().next() {
			for (key, option) in [
				("Archive", pkg_file.archive()),
				("Origin", pkg_file.origin()),
				("Codename", pkg_file.codename()),
				("Component", pkg_file.component()),
			] {
				if let Some(value) = option {
					map.insert(key, value.to_string());
				}
			}
		}

		map.insert("Provides", self.ver.provides().collect::<Vec<_>>().format());
		if let Some(desc) = self.ver.description() {
			map.insert("Description", desc);
		}

		let pkg = self.ver.parent();
		let mut attrs = vec![];
		if let Some(installed) = pkg.installed() {
			attrs.push("Installed".into());

			// Version isn't downloadable, consider it locally installed
			if !self.ver.is_downloadable() {
				attrs.push("Local".into());
			}

			if pkg.is_auto_removable() {
				attrs.push("Auto-Removable".into());
			}

			if pkg.is_auto_installed() {
				attrs.push("Automatic".into());
			}

			if let Some(candidate) = pkg.candidate() {
				// Version is installed, check if it's upgradable
				if self.ver == installed && self.ver < candidate {
					attrs.push(format!(
						"Upgradable to: {}",
						color::ver!(candidate.version())
					));
				}

				// This Version isn't installed, see if it's the candidate
				if self.ver == candidate && self.ver > installed {
					attrs.push(format!(
						"Upgradable from: {}",
						color::ver!(installed.version())
					));
				}
			}
		}

		map.insert("Attributes", format!("[{}]", attrs.join(", ")));

		map
	}

	pub fn pretty_map(&self) -> IndexMap<&str, String> {
		let mut map = self.map();

		for kind in DEP_ITER {
			if let Some(deps) = self.ver.get_depends(kind) {
				map.insert(kind.as_ref(), deps.format());
			}
		}

		// Package File Section
		if let Some(pkg_file) = self.ver.package_files().next() {
			map.insert("Origin", pkg_file.origin().unwrap_or("Unknown").to_string());

			// Check if source is local, pacstall or from a repo
			let mut source = String::new();
			if let Some(archive) = pkg_file.archive() {
				if archive == "now" {
					source += &format_local(self.ver.parent().name());
				} else {
					let uri = self.ver.uris().next().unwrap();
					source += URL.find(&uri).unwrap().as_str();
					source += &pkg_file.format();
				}
				map.insert("APT-Sources", source);
			}
		}
		map
	}

	pub fn show(&self, config: &Config) -> Result<()> {
		if config.get_bool("machine", false) {
			println!("{}", self.to_json()?);
			return Ok(());
		}

		for (key, value) in &self.pretty_map() {
			print_info(key, value);
		}

		Ok(())
	}

	/// List a single version of a package
	pub fn list(&self, config: &Config) -> Result<()> {
		if config.get_bool("machine", false) {
			println!("{}", self.to_json()?);
			return Ok(());
		}

		let mut string = self.ver.format();
		if let Some(pkg_file) = self.ver.package_files().next() {
			string += &pkg_file.format();
		}

		string += self.map().get("Attributes").unwrap();

		let description = config.get_bool("description", false);
		let summary = config.get_bool("summary", false);

		let desc = if description {
			self.ver
				.description()
				.unwrap_or_else(|| "No Description".to_string())
		} else if summary {
			self.ver
				.summary()
				.unwrap_or_else(|| "No Summary".to_string())
		} else {
			"".to_string()
		};

		if description || summary {
			string += "\n";
			string += &desc;
		}

		println!("{string}");
		Ok(())
	}

	pub fn to_json(&self) -> Result<String> { Ok(serde_json::to_string_pretty(&self.ver)?) }
}
