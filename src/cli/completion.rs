use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use clap_complete::engine::CompletionCandidate;
use rust_apt::{new_cache, Cache, PackageSort, PkgCurrentState};

const HISTORY_DEFAULT: &str = "/var/lib/nala/history";

fn candidates_from_names(names: impl IntoIterator<Item = String>) -> Vec<CompletionCandidate> {
	names.into_iter().map(CompletionCandidate::new).collect()
}

fn current_cache(current: &OsStr) -> Option<(&str, Cache)> {
	let current = current.to_str()?;
	let cache = new_cache!().ok()?;

	Some((current, cache))
}

pub fn package_completion(current: &OsStr) -> Vec<CompletionCandidate> {
	let Some((current, cache)) = current_cache(current) else {
		return Vec::new();
	};

	let names = cache
		.packages(&PackageSort::default().include_virtual().names())
		.map(|pkg| pkg.name().to_string())
		.filter(|name| name.starts_with(current))
		.collect::<BTreeSet<_>>();

	names.into_iter().map(CompletionCandidate::new).collect()
}

pub fn installed_package_completion(current: &OsStr) -> Vec<CompletionCandidate> {
	let Some((current, cache)) = current_cache(current) else {
		return Vec::new();
	};

	let names = cache
		.packages(&PackageSort::default().include_virtual().names())
		.filter(|pkg| {
			matches!(
				pkg.current_state(),
				PkgCurrentState::Installed
					| PkgCurrentState::HalfInstalled
					| PkgCurrentState::UnPacked
					| PkgCurrentState::HalfConfigured
					| PkgCurrentState::ConfigFiles
			)
		})
		.map(|pkg| pkg.name().to_string())
		.filter(|name| name.starts_with(current))
		.collect::<BTreeSet<_>>();

	names.into_iter().map(CompletionCandidate::new).collect()
}

pub fn history_id_completion(current: &OsStr) -> Vec<CompletionCandidate> {
	let Some(current) = current.to_str() else {
		return Vec::new();
	};

	let history_dir = std::env::var_os("NALA_HISTORY_DIR")
		.map(PathBuf::from)
		.unwrap_or_else(|| PathBuf::from(HISTORY_DEFAULT));

	let Ok(entries) = fs::read_dir(history_dir) else {
		return Vec::new();
	};

	let mut ids = BTreeSet::new();
	for entry in entries.flatten() {
		let path = entry.path();
		if !path.is_file() || path.extension().is_none_or(|ext| ext != "json") {
			continue;
		}

		let Some(id) = path.file_stem().and_then(|stem| stem.to_str()) else {
			continue;
		};

		if id.parse::<u32>().is_ok() && id.starts_with(current) {
			ids.insert(id.to_string());
		}
	}

	if "last".starts_with(current) {
		ids.insert("last".to_string());
	}

	candidates_from_names(ids)
}
