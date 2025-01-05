use anyhow::Result;
use rust_apt::Package;

use crate::cmd::ShowVersion;
use crate::config::{color, Config};

/// List packages in a vector
///
/// Shared function between list and search
pub fn list_packages(config: &Config, packages: Vec<Package>) -> Result<()> {
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
		// Fall back to the first version in the list if there isn't a candidate.
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
			"{} (Virtual) [{provides}]",
			color::primary!(pkg.fullname(true))
		);
	}

	Ok(())
}
