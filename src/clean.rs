use std::fs::{read_dir, remove_file};

use anyhow::{Context, Result};

use crate::config::{Config, Paths};

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
		let lists_dir = config.get_path(&Paths::Lists);
		remove_files(&lists_dir)?;
		return remove_files(&(lists_dir + "partial/"));
	}

	if config.get_bool("fetch", false) {
		let nala_sources = Paths::NalaSources.path();
		return remove_file(nala_sources)
			.with_context(|| format!("Failed to remove {nala_sources}"));
	}

	let archive = config.get_path(&Paths::Archive);
	remove_files(&archive)?;
	remove_files(&(archive + "partial/"))
}
