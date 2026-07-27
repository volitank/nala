use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::{Context, Result};
use rust_apt::tagfile::parse_tagfile;

use crate::config::{Config, Paths};
use crate::util::DOMAIN;
use crate::t;

fn domain_from_list(line: &str) -> Option<String> {
	if line.starts_with('#') || line.is_empty() {
		return None;
	}
	regex_string(line)
}

fn regex_string(line: &str) -> Option<String> {
	Some(DOMAIN.captures(line)?.get(1)?.as_str().to_string())
}

fn read_optional<T>(result: std::io::Result<T>, path: &Path) -> Result<Option<T>> {
	match result {
		Ok(value) => Ok(Some(value)),
		Err(err) if err.kind() == ErrorKind::NotFound => Ok(None),
		Err(err) => Err(err)
			.with_context(|| t!("file-read", "path" => path.display().to_string())),
	}
}

fn parse_list_sources(sources: &mut HashSet<String>, data: &str) {
	for line in data.lines() {
		if let Some(domain) = domain_from_list(line) {
			sources.insert(domain);
		}
	}
}

fn parse_deb822_sources(sources: &mut HashSet<String>, data: &str) -> Result<()> {
	for section in parse_tagfile(data)? {
		let enabled = section.get_default("Enabled", "yes").to_lowercase();

		// These sources are disabled. So we can ignore them
		if ["no", "false", "0"].contains(&enabled.as_str()) {
			continue;
		}

		let Some(uris) = section.get("URIs") else {
			continue;
		};

		for uri in uris.split_whitespace() {
			if uri.is_empty() {
				continue;
			}

			if let Some(domain) = regex_string(uri) {
				sources.insert(domain);
			}
		}
	}
	Ok(())
}

pub(super) fn parse_sources(config: &Config) -> Result<HashSet<String>> {
	let mut sources = HashSet::new();

	// Read and extract domains from the main sources.list file, if present.
	let main = config.get_path(&Paths::SourceList);
	if let Some(data) = read_optional(fs::read_to_string(&main), &main)? {
		parse_list_sources(&mut sources, &data);
	}

	// Parts could be either .list or .sources
	let parts = config.get_path(&Paths::SourceParts);
	let Some(entries) = read_optional(fs::read_dir(&parts), &parts)? else {
		return Ok(sources);
	};

	for file in entries {
		let path = file?.path();
		if path.is_dir() {
			continue;
		}

		let filename = path.to_string_lossy();

		// Don't consider nala sources as it'll be overwritten
		if filename.ends_with("nala.sources") {
			continue;
		}

		// Continue if the file isn't .sources or .list
		if !filename.ends_with(".sources") && !filename.ends_with(".list") {
			continue;
		}

		let data = fs::read_to_string(&path)
			.with_context(|| t!("file-read", "path" => path.display().to_string()))?;

		if filename.ends_with(".sources") {
			parse_deb822_sources(&mut sources, &data)?;
			continue;
		}

		if filename.ends_with(".list") {
			parse_list_sources(&mut sources, &data);
		}
	}
	Ok(sources)
}
