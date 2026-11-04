use anyhow::Result;
use rust_apt::new_cache;

use crate::cmd::ShowVersion;
use crate::config::{Config, color};
use crate::{glob, t};

/// List or search packages selected from the cache.
pub fn list(config: &Config) -> Result<()> {
	let cache = new_cache!()?;
	let mut missing = Vec::new();
	let packages = if config.command == "search" {
		glob::regex_pkgs(config, &cache)?
	} else {
		match config.pkg_names() {
			Ok(names) => {
				let selection = glob::pkgs_with_modifiers(names, config, &cache)?;
				let (packages, selection_missing) = selection.into_packages_and_missing();
				missing = selection_missing;
				packages
			},
			Err(_) => glob::sorted_pkgs(config, &cache),
		}
	};

	// We at least have one package so we can begin listing.
	let all_versions = config.get_bool("all_versions", false);
	for pkg in packages {
		if all_versions && pkg.has_versions() {
			for version in pkg.versions() {
				ShowVersion::new(version).list(config)?;
			}
			continue;
		}

		// Get the candidate if we're only going to show one version.
		// Fall back to the first version in the list if there isn't a
		// candidate.
		if let Some(version) = pkg.candidate().or(pkg.versions().next()) {
			// There is a version! Let's format it
			ShowVersion::new(version).list(config)?;
			continue;
		}

		// There are no versions so it must be a virtual package
		let provides = pkg
			.provides()
			.map(|p| p.package().fullname(true))
			.collect::<Vec<_>>()
			.join(", ");

		println!(
			"{} ({}) [{provides}]",
			color::primary!(pkg.fullname(true)),
			t!("list-virtual")
		);
	}

	glob::log_missing_notices(&missing);
	Ok(())
}
