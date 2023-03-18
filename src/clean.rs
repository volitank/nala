use std::fs::{read_dir, remove_file};

use anyhow::{Context, Result};

use crate::config::Config;

fn remove_files(file_str: &str) -> Result<()> {
	// If the path doesn't exist just ignore it
	if let Ok(paths) = read_dir(file_str) {
		// Flatten the errors away!
		for path in paths.flatten() {
			if let Ok(is_file) = path.file_type() {
				if is_file.is_file() {
					remove_file(path.path())
						.with_context(|| format!("Failed to remove {}", path.path().display()))?;
				}
			}
		}
	}
	Ok(())
}

pub fn clean(config: &Config) -> Result<()> {
	if config.get_bool("lists", false) {
		remove_files("/var/lib/apt/lists/")?;
		return remove_files("/var/lib/apt/lists/partial/");
	}

	if config.get_bool("fetch", false) {
		remove_file("/etc/apt/sources.list.d/nala-sources.list")
			.context("Failed to remove `/etc/apt/sources.list.d/nala-sources.list`")?;
		// Because of anyhow we have to return Ok here.
		return Ok(());
	}

	remove_files("/var/cache/apt/archives/")?;
	remove_files("/var/cache/apt/lists/partial/")
}
