use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use rust_apt::new_cache;

use crate::config::{color, Config, Theme};
use crate::deb::DebFile;
use crate::util::PACSTALL;
use crate::{glob, info, t};

use super::ShowVersion;

pub fn format_local(pkg_name: &str) -> String {
	// Check if this could potentially be a Pacstall Package.
	let mut pac_repo = String::new();
	let postfixes = ["", "-deb", "-git", "-bin", "-app"];
	for postfix in postfixes {
		if let Ok(metadata) = fs::read_to_string(format!(
			"/var/log/pacstall/metadata/{pkg_name}{postfix}"
		)) {
			if let Some(repo) = PACSTALL.captures(&metadata) {
				pac_repo += repo.get(1).unwrap().as_str();
			} else {
				pac_repo += "https://github.com/pacstall/pacstall-programs";
			}
		}
	}
	if pac_repo.is_empty() {
		return t!("show-local-install");
	}

	color::secondary!(pac_repo)
}

/// The show command
pub async fn show(config: &Config) -> Result<()> {
	let mut names = vec![];
	let mut debs = vec![];
	for name in config.pkg_names()? {
		if name.ends_with(".deb") {
			debs.push(
				DebFile::new(name.clone())
					.await
					.with_context(|| t!("file-read", "path" => &name))?,
			);
		} else {
			names.push(name);
		}
	}
	let paths: Vec<&str> = debs.iter().map(|deb| deb.path.as_str()).collect();
	let cache = new_cache!(&paths)?;
	for deb in &debs {
		let arch = deb.control.get("Architecture").map_or("all", String::as_str);
		let name = if arch == "all" {
			deb.name().to_string()
		} else {
			format!("{}:{arch}", deb.name())
		};
		let pkg = cache
			.get(&name)
			.with_context(|| t!("pkg-not-found", "package" => &deb.path))?;
		let path = fs::canonicalize(&deb.path)?;
		let selected = pkg.versions().find_map(|version| {
			if version.version() != deb.version() {
				return None;
			}
			let file = version.version_files().find(|file| {
				file.package_file()
					.filename()
					.and_then(|name| Path::new(name).canonicalize().ok())
					.is_some_and(|name| name == path)
			})?;
			Some((version, file))
		});
		let (version, file) = selected.with_context(|| {
			t!("pkg-version-missing", "package" => &name, "version" => deb.version())
		})?;
		ShowVersion::from_file(version, Some(file)).show(config)?;
	}

	let mut additional_records = 0;
	// Filter virtual packages into their real package.
	let all_versions = config.get_bool("all_versions", false);
	let selection = glob::pkgs_with_modifiers(names, config, &cache)?;
	let (packages, missing) = selection.into_packages_and_missing();
	for pkg in &packages {
		let versions = pkg.versions().map(ShowVersion::new).collect::<Vec<_>>();
		additional_records += versions.len();

		if all_versions {
			for version in &versions {
				version.show(config)?;
				additional_records -= 1;
			}
		} else if let Some(version) = versions.first() {
			version.show(config)?;
			additional_records -= 1;
		}
	}

	if additional_records != 0 {
		let count = color::color!(Theme::Notice, &additional_records.to_string());
		let switch = color::color!(Theme::Notice, "'-a'");
		info!(
			"{}",
			t!("show-more-records", "count" => count, "switch" => switch),
		);
	}

	glob::log_missing_notices(&missing);

	Ok(())
}
