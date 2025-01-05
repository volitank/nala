use std::fs;

use anyhow::Result;
use rust_apt::new_cache;

use crate::config::{color, Config, Theme};
use crate::{glob, info};
use crate::util::PACSTALL;

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
		return "local install".to_string();
	}

	color::secondary!(pac_repo).into()
}

/// The show command
pub fn show(config: &Config) -> Result<()> {
	let cache = new_cache!()?;

	let mut additional_records = 0;
	// Filter virtual packages into their real package.
	let all_versions = config.get_bool("all_versions", false);
	let packages = glob::pkgs_with_modifiers(config.pkg_names()?, config, &cache)?.only_pkgs();
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
		info!(
			"There are {} additional records. Please use the {} switch to see them.",
			color::color!(Theme::Notice, &additional_records.to_string()),
			color::color!(Theme::Notice, "'-a'"),
		);
	}

	Ok(())
}
