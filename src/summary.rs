use std::collections::HashSet;
use std::io::Write;

use anyhow::{bail, Result};
use chrono::Utc;
use rust_apt::util::DiskSpace;
use rust_apt::{Cache, Package};

use crate::config::{color, Config, Paths, Theme};
use crate::download::Downloader;
use crate::libnala::{apt_hook_with_pkgs, get_history, run_scripts, HistoryEntry, NalaCache};
use crate::{dpkg, error, table, tui, warn};

/// Ask the user a question and let them decide Y or N
pub fn ask(msg: &str) -> Result<()> {
	print!("{msg} [Y/n] ");
	std::io::stdout().flush()?;

	let mut response = String::new();
	std::io::stdin().read_line(&mut response)?;

	let resp = response.to_lowercase();
	if resp.starts_with('y') {
		return Ok(());
	}

	if resp.starts_with('n') {
		bail!("User refused confirmation")
	}

	bail!("'{}' is not a valid response", response.trim())
}

/// TODO: Implement a simple summary that is very short for serial/console users
pub async fn display_summary(
	cache: &Cache,
	config: &Config,
	history: &HistoryEntry,
) -> Result<bool> {
	if config.get_no_bool("tui", true) {
		// App returns true if we should continue.
		tui::summary::SummaryTab::new(cache, config, history)
			.run()
			.await
	} else {
		let mut tables = vec![];
		let map = history.to_map();
		for (op, pkgs) in map.iter() {
			let mut table = table::get_table(if pkgs[0].items(config).len() > 3 {
				&["Package:", "Old Version:", "New Version:", "Size:"]
			} else {
				&["Package:", "Version:", "Size:"]
			});

			table.add_rows(pkgs.iter().map(|p| p.items(config)));
			tables.push((op, table));
		}

		let width = rust_apt::util::terminal_width();
		let sep = "=".repeat(width);

		for (op, pkgs) in tables {
			println!("{sep}");
			println!(" {}", color::highlight!(op.as_str()));
			println!("{sep}");

			println!("{pkgs}");
		}
		println!("{sep}");
		println!(" Summary");
		println!("{sep}");

		for (op, pkgs) in map {
			println!(" {op} {}", pkgs.len())
		}

		println!();
		if cache.depcache().download_size() > 0 {
			println!(
				" Total download size: {}",
				config.unit_str(cache.depcache().download_size())
			)
		}

		match cache.depcache().disk_size() {
			DiskSpace::Require(disk_space) => {
				println!(" Disk space required: {}", config.unit_str(disk_space))
			},
			DiskSpace::Free(disk_space) => {
				println!(" Disk space to free: {}", config.unit_str(disk_space))
			},
		}
		println!();

		// Returns an error if yes is no selected
		ask("Do you want to continue?")?;
		Ok(true)
	}
}

fn check_essential(config: &Config, pkgs: &Vec<Package>) -> Result<()> {
	let essential = pkgs
		.iter()
		.filter(|p| p.is_essential() && p.marked_delete())
		.collect::<Vec<_>>();

	if essential.is_empty() {
		return Ok(());
	}

	warn!("The following packages are essential!");
	eprintln!(
		"  {}",
		essential
			.iter()
			.map(|p| p.name())
			.collect::<Vec<_>>()
			.join(", ")
	);

	if config.get_bool("remove_essential", false) {
		return Ok(());
	}

	error!("You have attempted to remove essential packages");

	let switch = color::color!(Theme::Warning, "--remove-essential");
	bail!("Use '{switch}' if you are sure.")
}

pub async fn commit(cache: Cache, config: &Config) -> Result<()> {
	// Package is not really mutable in the way clippy thinks.
	#[allow(clippy::mutable_key_type)]
	let auto = if config.get_no_bool("auto_remove", true) {
		let purge = config.get_bool("purge", false);
		let remove_config = config.get_bool("remove_config", false);
		cache.auto_remove(remove_config, purge)
	} else {
		HashSet::new()
	};

	let (pkgs, pkg_set) = cache.sort_changes(auto)?;
	check_essential(config, &pkgs)?;

	if pkg_set.is_empty() {
		println!("Nothing to do.");
		return Ok(());
	}

	let versions = pkgs
		.iter()
		.filter_map(|p| p.install_version())
		.collect::<Vec<_>>();

	let mut downloader = Downloader::new(config)?;
	let archive = config.get_path(&Paths::Archive);

	for ver in &versions {
		downloader.add_version(ver, &archive).await?;
	}

	if config.get_bool("print_uris", false) {
		for uri in downloader.uris() {
			println!("{}", uri.to_json()?);
		}
		// Print uris does not go past here
		return Ok(());
	};

	let history_entry = HistoryEntry::new(
		get_history(config)
			.await?
			.iter()
			.map(|entry| entry.id)
			.max()
			.unwrap_or_default()
			+ 1,
		Utc::now().to_rfc3339(),
		pkg_set.into_values().flatten().collect(),
	);

	if !crate::summary::display_summary(&cache, config, &history_entry).await? {
		return Ok(());
	};

	// Only download if needed
	// Downloader will error if empty download
	// TODO: Should probably just make run check and return Ok(vec![])?
	if !downloader.uris().is_empty() {
		let _finished = downloader.run(config, false).await?;
	}

	if config.get_bool("download_only", false) {
		return Ok(());
	}

	history_entry.write_to_file(config).await?;

	// TODO: There should likely be a field in the history
	// to mark that it was a transaction that failed.
	// The idea is to run the rest of this program,
	// catch any errors, and then write the history file
	// Either way but we'll know that it failed.

	run_scripts(config, "DPkg::Pre-Invoke")?;
	apt_hook_with_pkgs(config, &pkgs, "DPkg::Pre-Install-Pkgs")?;

	config.apt.set("Dpkg::Use-Pty", "0");

	dpkg::run_install(cache, config)?;

	run_scripts(config, "DPkg::Post-Invoke")
}
