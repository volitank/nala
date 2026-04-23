use std::collections::{HashMap, HashSet};

use anyhow::{bail, Result};
use chrono::Utc;
use rust_apt::util::DiskSpace;
use rust_apt::{Cache, Package};

use crate::cmd::{self, apt_hook_with_pkgs, ask, run_scripts, HistoryEntry};
use crate::config::{color, Config, Paths, Theme};
use crate::download::Downloader;
use crate::libnala::{NalaCache, Operation, PackageTransition};
use crate::terminal::{use_tui, TerminalGuard};
use crate::tui::summary::SummaryRow;
use crate::{dpkg, error, table, tui, warn};

/// TODO: Implement a simple summary that is very short for serial/console users
pub async fn display_summary(
	cache: &Cache,
	config: &Config,
	pkg_set: &HashMap<Operation, Vec<PackageTransition>>,
) -> Result<bool> {
	if use_tui(config) {
		// App returns true if we should continue.
		let mut terminal = TerminalGuard::new()?;
		tui::summary::SummaryTab::new(cache, config, pkg_set)
			.run(&mut terminal)
			.await
	} else {
		let mut tables = vec![];
		for (op, pkgs) in pkg_set {
			let rows = pkgs.iter().map(SummaryRow::new).collect::<Vec<_>>();
			let mut table = table::get_table(if rows[0].items(config).len() > 3 {
				&["Package:", "Old Version:", "New Version:", "Size:"]
			} else {
				&["Package:", "Version:", "Size:"]
			});

			table.add_rows(rows.iter().map(|row| row.items(config)));
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

		for (op, pkgs) in pkg_set {
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

fn collect_history_packages(
	pkg_set: HashMap<Operation, Vec<PackageTransition>>,
) -> Vec<PackageTransition> {
	pkg_set
		.into_iter()
		.filter(|(operation, _)| *operation != Operation::Held)
		.flat_map(|(_, packages)| packages)
		.collect()
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
		if ver
			.uris()
			.next()
			.is_some_and(|uri| !uri.starts_with("file:"))
		{
			downloader.add_version(ver, &archive).await?;
		}
	}

	if config.get_bool("print_uris", false) {
		for uri in downloader.uris() {
			println!("{}", uri.to_json()?);
		}
		// Print uris does not go past here
		return Ok(());
	};

	if !crate::summary::display_summary(&cache, config, &pkg_set).await? {
		return Ok(());
	};

	let started_at = Utc::now().to_rfc3339();

	// Only download if needed
	// Downloader will error if empty download
	// TODO: Should probably just make run check and return Ok(vec![])?
	if !downloader.uris().is_empty() {
		let _finished = downloader.run(config, false).await?;
	}

	if config.get_bool("download_only", false) {
		return Ok(());
	}

	// TODO: There should likely be a field in the history
	// to mark that it was a transaction that failed.
	// The idea is to run the rest of this program,
	// catch any errors, and then write the history file
	// Either way but we'll know that it failed.

	run_scripts(config, "DPkg::Pre-Invoke")?;
	apt_hook_with_pkgs(config, &pkgs, "DPkg::Pre-Install-Pkgs")?;

	config.apt.set("Dpkg::Use-Pty", "0");

	dpkg::run_install(cache, config)?;

	let history_packages = collect_history_packages(pkg_set);
	if !history_packages.is_empty() {
		let history_entry = HistoryEntry::applied(
			config,
			cmd::next_history_id(config).await?,
			started_at,
			Utc::now().to_rfc3339(),
			history_packages,
		);

		history_entry.write_to_file(config)?;
	}

	run_scripts(config, "DPkg::Post-Invoke")
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::libnala::PackageState;

	#[test]
	fn collect_history_packages_skips_non_transaction_rows() {
		let mut pkg_set = HashMap::new();
		pkg_set.insert(
			Operation::Install,
			vec![PackageTransition::transition(
				"demo".to_string(),
				1,
				Operation::Install,
				PackageState::missing(),
				PackageState::config_only(Some("1.0".to_string()), Some(false)),
			)],
		);
		pkg_set.insert(
			Operation::Held,
			vec![PackageTransition::transition(
				"held-demo".to_string(),
				1,
				Operation::Held,
				PackageState::missing(),
				PackageState::config_only(Some("2.0".to_string()), Some(false)),
			)],
		);

		let packages = collect_history_packages(pkg_set);

		assert_eq!(packages.len(), 1);
		assert_eq!(packages[0].name, "demo");
		assert_eq!(packages[0].operation, Operation::Install);
	}
}
