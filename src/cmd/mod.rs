macro_rules! define_modules {
	($($module:ident),*) => {
		$(
			mod $module;
			pub use $module::$module;
		)*
	};
}

define_modules!(show, policy, update, upgrade, history, fetch, clean);

pub mod install;
mod list;
pub mod traits;

use anyhow::Result;
pub use history::{HistoryEntry, next_history_id};
use indexmap::IndexMap;
pub use install::{fix_broken, mark_cli_pkgs};
pub use list::list_packages;
use rust_apt::records::RecordField;
use rust_apt::{DepType, Version};
use show::format_local;
use traits::ShowFormat;
pub use upgrade::{apt_hook_with_pkgs, run_scripts};

use crate::cli::commands::Moo;
use crate::config::{Config, color};
pub use crate::libnala::Operation;
use crate::t;
use crate::util::URL;

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

fn show_label(key: &str) -> String {
	match key {
		RecordField::Package => t!("show-package"),
		RecordField::Version => t!("show-version"),
		RecordField::Architecture => t!("show-architecture"),
		RecordField::Priority => t!("show-priority"),
		RecordField::Essential => t!("show-essential"),
		RecordField::Section => t!("show-section"),
		RecordField::Source => t!("show-source"),
		RecordField::InstalledSize => t!("show-installed-size"),
		RecordField::Size => t!("show-size"),
		RecordField::Maintainer => t!("show-maintainer"),
		RecordField::OriginalMaintainer => t!("show-original-maintainer"),
		RecordField::Homepage => t!("show-homepage"),
		RecordField::SHA256 => t!("show-sha256"),
		"Archive" => t!("show-archive"),
		"Origin" => t!("show-origin"),
		"Codename" => t!("show-codename"),
		"Component" => t!("show-component"),
		"Provides" => t!("show-provides"),
		"Description" => t!("show-description"),
		"Attributes" => t!("show-attributes"),
		"APT-Sources" => t!("show-apt-sources"),
		"Depends" => t!("show-depends"),
		"PreDepends" => t!("show-pre-depends"),
		"Suggests" => t!("show-suggests"),
		"Recommends" => t!("show-recommends"),
		"Conflicts" => t!("show-conflicts"),
		"Replaces" => t!("show-replaces"),
		"Obsoletes" => t!("show-obsoletes"),
		"Breaks" | "DpkgBreaks" => t!("show-breaks"),
		"Enhances" => t!("show-enhances"),
		_ => key.to_string(),
	}
}

pub(crate) struct ShowVersion<'a> {
	ver: Version<'a>,
	records: IndexMap<&'static str, String>,
}

impl ShowVersion<'_> {
	pub fn new(ver: Version) -> ShowVersion {
		let records = IndexMap::from_iter(
			RECORDS
				.iter()
				.copied()
				.map(|key| (key, ver.get_record(key).unwrap_or_else(|| t!("unknown")))),
		);
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
			attrs.push(t!("show-attr-installed"));

			// Version isn't downloadable, consider it locally installed
			if !self.ver.is_downloadable() {
				attrs.push(t!("show-attr-local"));
			}

			if pkg.is_auto_removable() {
				attrs.push(t!("show-attr-auto-removable"));
			}

			if pkg.is_auto_installed() {
				attrs.push(t!("show-attr-automatic"));
			}

			if let Some(candidate) = pkg.candidate() {
				// Version is installed, check if it's upgradable
				if self.ver == installed && self.ver < candidate {
					attrs.push(t!(
						"show-attr-upgradable-to",
						"version" => color::ver!(candidate.version()),
					));
				}

				// This Version isn't installed, see if it's the candidate
				if self.ver == candidate && self.ver > installed {
					attrs.push(t!(
						"show-attr-upgradable-from",
						"version" => color::ver!(installed.version()),
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
			map.insert(
				"Origin",
				pkg_file
					.origin()
					.map(str::to_string)
					.unwrap_or_else(|| t!("unknown")),
			);

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
		if config.get_bool(crate::config::keys::MACHINE, false) {
			println!("{}", self.to_json()?);
			return Ok(());
		}

		for (key, value) in &self.pretty_map() {
			let header = show_label(key);
			print_info(&header, value);
		}

		Ok(())
	}

	/// List a single version of a package
	pub fn list(&self, config: &Config) -> Result<()> {
		if config.get_bool(crate::config::keys::MACHINE, false) {
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
				.unwrap_or_else(|| t!("show-no-description"))
		} else if summary {
			self.ver.summary().unwrap_or_else(|| t!("show-no-summary"))
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

const CAT: &str = r#"
   /\_/\    (`\
  (='.'=).--.) )
  (")_(")----\/"#;

pub fn moo(moo: Moo) -> Result<()> {
	if moo.help {
		println!("I beg, pls moo");
		return Ok(());
	}
	println!("{CAT}");

	println!("\"...I can't moo for I'm a cat...\"");

	if moo.update {
		println!("\"...What did you expect to update to do?...\"");
	} else if moo.no_update {
		println!("\"...What did you expect no-update to do?...\"");
	}

	Ok(())
}
