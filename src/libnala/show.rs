use anyhow::Result;
use indexmap::{IndexMap, IndexSet};
use rust_apt::records::RecordField;
use rust_apt::{BaseDep, DepType, Dependency, PackageFile, Provider, Version};

use crate::config::{color, Config, Theme};
use crate::libnala::{PACSTALL, URL};

pub trait ShowFormat {
	fn format(&self) -> String;
}

pub struct ShowVersion<'a> {
	ver: Version<'a>,
	records: IndexMap<&'static str, String>,
}

const RECORDS: [&str; 13] = [
	// Status: install ok installed
	// Priority: optional
	// Section: utils
	// Installed-Size: 2247
	// Maintainer: RPM packaging team <team+pkg-rpm@tracker.debian.org>
	// Architecture: amd64
	// Multi-Arch: foreign
	// Source: libzstd
	// Version: 1.5.6+dfsg-1
	// Depends: libc6 (>= 2.34), libgcc-s1 (>= 3.0), liblz4-1 (>= 1.8.0), liblzma5 (>=
	// 5.1.1alpha+20120614), libstdc++6 (>= 12), zlib1g (>= 1:1.1.4) Description: fast lossless
	// compression algorithm -- CLI tool  Zstd, short for Zstandard, is a fast lossless
	// compression algorithm, targeting  real-time compression scenarios at zlib-level compression
	// ratio.  .
	//  This package contains the CLI program implementing zstd.
	// Homepage: https://github.com/facebook/zstd
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

impl ShowFormat for BaseDep<'_> {
	fn format(&self) -> String {
		// These Dependency types will be colored red
		let theme = if matches!(self.dep_type(), DepType::Conflicts | DepType::DpkgBreaks) {
			Theme::Error
		} else {
			Theme::Primary
		};

		if let Some(comp) = self.comp_type() {
			return format!(
				// libgnutls30 (>= 3.7.5)
				"{} {}{comp} {}{}",
				// There's a compare operator in the dependency.
				// Dang better have a version smh my head.
				color::color!(theme, self.target_package().name()),
				color::highlight!("("),
				color::color!(Theme::Secondary, self.version().unwrap()),
				color::highlight!(")"),
			);
		}
		color::color!(theme, self.target_package().name()).into()
	}
}

const DEP_BUFFER: &str = "\n    ";
const DEP_SEP: &str = " | ";
impl ShowFormat for &Vec<Dependency<'_>> {
	fn format(&self) -> String {
		let mut depends_string = String::new();
		// Get total deps number to include Or Dependencies
		let total_deps = self.len();

		// If there are more than 4 deps format with multiple lines
		if total_deps > 3 {
			depends_string += DEP_BUFFER;
		}

		let mut inner = IndexSet::new();
		for (i, dep) in self.iter().enumerate() {
			let target = dep.first().target_package().name();
			if inner.contains(target) {
				continue;
			}
			inner.insert(target);

			// Or Deps need to be formatted slightly different.
			if dep.is_or() {
				for (j, base_dep) in dep.iter().enumerate() {
					depends_string += &base_dep.format();
					if j + 1 != dep.len() {
						depends_string += DEP_SEP;
					}
				}
			} else {
				// Regular dependencies are more simple than Or
				depends_string += &dep.first().format();
			}

			depends_string += if total_deps > 3 {
				DEP_BUFFER
			// Only add the comma if it isn't the last.
			} else if i + 1 != total_deps {
				", "
			} else {
				" "
			};
		}
		depends_string.trim_end().to_string()
	}
}

impl ShowFormat for PackageFile<'_> {
	fn format(&self) -> String {
		let mut string = String::new();

		let Some(archive) = self.archive() else {
			return "ERROR:?".into();
		};

		if archive == "now" {
			return " [now]".into();
		}

		string += " [";
		for (key, postfix) in [
			(self.origin(), "/"),
			(self.codename(), " "),
			(self.component(), "] "),
		] {
			if let Some(value) = key {
				string += value;
			}
			string += postfix;
		}
		string
	}
}

impl ShowFormat for Vec<Provider<'_>> {
	fn format(&self) -> String {
		format!(
			"[{}]",
			self.iter()
				.map(|p| p.name())
				.collect::<Vec<&str>>()
				.join(", ")
		)
	}
}

impl ShowFormat for Version<'_> {
	fn format(&self) -> String {
		format!(
			"{} {}",
			color::primary!(&self.parent().fullname(true)),
			color::ver!(self.version()),
		)
	}
}

pub fn format_local(pkg_name: &str) -> String {
	// Check if this could potentially be a Pacstall Package.
	let mut pac_repo = String::new();
	let postfixes = ["", "-deb", "-git", "-bin", "-app"];
	for postfix in postfixes {
		if let Ok(metadata) =
			std::fs::read_to_string(format!("/var/log/pacstall/metadata/{pkg_name}{postfix}"))
		{
			if let Some(repo) = PACSTALL.captures(&metadata) {
				pac_repo += repo.get(1).unwrap().as_str();
			} else {
				pac_repo += "https://github.com/pacstall/pacstall-programs";
			}
		}
	}
	if pac_repo.is_empty() {
		return "local install".to_string();
	}

	color::secondary!(pac_repo).into()
}

fn print_info(header: &str, value: &str) {
	let sep = color::highlight!(":");
	let header = color::highlight!(header);
	println!("{header}{sep} {value}")
}
