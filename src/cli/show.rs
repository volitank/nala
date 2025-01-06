use anyhow::Result;
use rust_apt::new_cache;

use crate::config::{color, Config, Theme};
use crate::libnala::ShowVersion;
use crate::{glob, info};

/// The show command
pub fn show(config: &Config) -> Result<()> {
	let cache = new_cache!()?;

	let mut additional_records = 0;
	// Filter virtual packages into their real package.
	let all_versions = config.get_bool("all_versions", false);
	let packages = glob::pkgs_with_modifiers(config.pkg_names()?, config, &cache)?.found();
	for pkg in packages {
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
