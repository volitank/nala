mod held;
mod hooks;

use std::collections::HashSet;

use anyhow::{bail, Result};
use rust_apt::cache::Upgrade;
use rust_apt::{new_cache, Cache};
pub use hooks::{apt_hook_with_pkgs, run_scripts};

use crate::config::{color, keys, Config};
use crate::libnala::{package_key, PackageKey};
use crate::util::sudo_check;
use crate::{debug, glob, info};

/// Executes an upgrade transaction after applying the selected APT upgrade
/// mode to a fresh cache.
pub async fn upgrade(config: &Config, upgrade_type: Upgrade) -> Result<()> {
	sudo_check(config)?;
	let cache = new_cache!()?;
	let held_snapshots = held::snapshots(&cache);
	let protected = protect_excluded_packages(&cache, config)?;

	debug!("Running Upgrade: {upgrade_type:?}");
	if let Err(err) = cache.upgrade(upgrade_type) {
		if !protected.is_empty() {
			bail!("Selected packages cannot be excluded from upgrade safely.\n{err}");
		}
		bail!(err);
	}

	crate::summary::commit_with_display_rows(cache, config, &protected, |changed| {
		held::transitions(held_snapshots, changed, &protected)
	})
	.await
}

fn protect_excluded_packages(cache: &Cache, config: &Config) -> Result<HashSet<PackageKey>> {
	let Some(excludes) = config.get_vec(keys::EXCLUDE).filter(|items| !items.is_empty()) else {
		return Ok(HashSet::new());
	};

	let mut protected = HashSet::new();
	let packages = glob::pkgs_matching_name_patterns(excludes, cache)?;

	for pkg in packages {
		let reason = if pkg.is_upgradable() {
			Some("upgrade")
		} else if pkg.is_auto_removable() {
			Some("auto-removal")
		} else {
			None
		};

		let Some(reason) = reason else {
			continue;
		};

		info!(
			"Protecting {} from {reason}",
			color::primary!(pkg.fullname(true))
		);
		cache.resolver().protect(&pkg);
		pkg.mark_keep();
		protected.insert(package_key(&pkg));
	}

	Ok(protected)
}
