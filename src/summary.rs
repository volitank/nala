use std::collections::{HashMap, HashSet};

use anyhow::{bail, Result};
use chrono::Utc;
use rust_apt::util::DiskSpace;
use rust_apt::{Cache, Package};

use crate::cmd::{self, apt_hook_with_pkgs, run_scripts, HistoryEntry};
use crate::config::{color, keys, Config, Paths, Theme};
use crate::download::Downloader;
use crate::libnala::{package_key, NalaCache, Operation, PackageKey, PackageTransition};
use crate::terminal::{use_tui, TerminalGuard};
use crate::tui::summary::SummaryRow;
use crate::{dpkg, error, info, table, tui, util, warn};

pub async fn display_summary(
	cache: &Cache,
	config: &Config,
	pkg_set: &HashMap<Operation, Vec<PackageTransition>>,
) -> Result<bool> {
	if config.simple_summary() {
		print_simple_summary(cache, config, pkg_set);
		util::confirm(config, "Do you want to continue?")?;
		return Ok(true);
	}

	if use_tui(config)
		&& !config.get_bool(keys::ASSUME_YES, false)
		&& !config.get_bool(keys::ASSUME_NO, false)
	{
		// App returns true if we should continue.
		let mut terminal = TerminalGuard::new()?;
		return tui::summary::SummaryTab::new(cache, config, pkg_set)
			.run(&mut terminal)
			.await;
	}

	print_full_summary(cache, config, pkg_set);
	util::confirm(config, "Do you want to continue?")?;
	Ok(true)
}

fn print_readonly_summary(
	cache: &Cache,
	config: &Config,
	pkg_set: &HashMap<Operation, Vec<PackageTransition>>,
) {
	if config.simple_summary() {
		print_simple_summary(cache, config, pkg_set);
	} else {
		print_full_summary(cache, config, pkg_set);
	}
}

fn sorted_summary_sets(
	pkg_set: &HashMap<Operation, Vec<PackageTransition>>,
) -> Vec<(Operation, &[PackageTransition])> {
	Operation::to_vec()
		.into_iter()
		.filter_map(|op| {
			pkg_set
				.get(&op)
				.filter(|packages| !packages.is_empty())
				.map(|packages| (op, packages.as_slice()))
		})
		.collect()
}

fn print_size_summary(cache: &Cache, config: &Config) {
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
}

fn print_simple_summary(
	cache: &Cache,
	config: &Config,
	pkg_set: &HashMap<Operation, Vec<PackageTransition>>,
) {
	let sets = sorted_summary_sets(pkg_set);
	for (op, pkgs) in &sets {
		let header = color::highlight!(op.as_str());
		println!("{header}: {}", pkgs.len());
		println!(
			"  {}",
			pkgs.iter()
				.map(|pkg| {
					pkg.held_reason.as_ref().map_or_else(
						|| pkg.name.clone(),
						|reason| format!("{} ({})", pkg.name, reason.summary()),
					)
				})
				.collect::<Vec<_>>()
				.join(", ")
		)
	}
	print_size_summary(cache, config);
}

fn print_full_summary(
	cache: &Cache,
	config: &Config,
	pkg_set: &HashMap<Operation, Vec<PackageTransition>>,
) {
	let mut tables = vec![];
	for (op, pkgs) in sorted_summary_sets(pkg_set) {
		let rows = pkgs.iter().map(SummaryRow::new).collect::<Vec<_>>();
		let mut table = table::get_table(&rows[0].headers());

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

	for (op, pkgs) in sorted_summary_sets(pkg_set) {
		println!(" {op} {}", pkgs.len())
	}

	print_size_summary(cache, config);
}

fn add_display_rows(
	pkg_set: &mut HashMap<Operation, Vec<PackageTransition>>,
	pkgs: &[Package<'_>],
	display_rows: impl FnOnce(&HashSet<PackageKey>) -> Vec<PackageTransition>,
) {
	let changed = pkgs.iter().map(package_key).collect::<HashSet<_>>();
	for package in display_rows(&changed) {
		pkg_set.entry(package.operation).or_default().push(package);
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

	if config.get_bool(keys::REMOVE_ESSENTIAL, false) {
		return Ok(());
	}

	error!("You have attempted to remove essential packages");

	let switch = color::color!(Theme::Warning, "--remove-essential");
	bail!("Use '{switch}' if you are sure.")
}

pub async fn commit(cache: Cache, config: &Config) -> Result<()> {
	commit_with_display_rows(cache, config, &HashSet::new(), |_| Vec::new()).await
}

pub(crate) async fn commit_with_display_rows(
	cache: Cache,
	config: &Config,
	protected: &HashSet<PackageKey>,
	display_rows: impl FnOnce(&HashSet<PackageKey>) -> Vec<PackageTransition>,
) -> Result<()> {
	// Package is not really mutable in the way clippy thinks.
	#[allow(clippy::mutable_key_type)]
	let auto = if config.get_no_bool(keys::AUTO_REMOVE, true) {
		let purge = config.get_bool("purge", false);
		let remove_config = config.get_bool("remove_config", false);
		cache.auto_remove(remove_config, purge, protected)
	} else {
		HashSet::new()
	};

	let (pkgs, mut pkg_set) = cache.sort_changes(auto)?;
	add_display_rows(&mut pkg_set, &pkgs, display_rows);
	check_essential(config, &pkgs)?;

	if pkgs.is_empty() {
		if pkg_set.is_empty() {
			println!("Nothing to do.");
		} else {
			print_readonly_summary(&cache, config, &pkg_set);
		}
		return Ok(());
	}

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

	let history_packages = pkg_set.into_values().flatten().collect::<Vec<_>>();
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

	run_scripts(config, "DPkg::Post-Invoke")?;

	check_reboot_required(config);

	Ok(())
}

fn check_reboot_required(config: &Config) {
	let reboot_path = config.get_path(&Paths::RebootRequired);
	if !reboot_path.exists() {
		return;
	}

	info!("A reboot is required to complete these changes.");

	let pkgs_path = config.get_path(&Paths::RebootRequiredPkgs);
	if pkgs_path.exists() {
		if let Ok(content) = std::fs::read_to_string(&pkgs_path) {
			let pkgs: Vec<&str> = content.lines().filter(|l| !l.is_empty()).collect();
			if !pkgs.is_empty() {
				info!("The following packages require a reboot:");
				info!(" {}", pkgs.join(", "));
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::libnala::PackageState;

	fn transition(name: &str, operation: Operation) -> PackageTransition {
		PackageTransition::transition(
			name.to_string(),
			1,
			operation,
			PackageState::missing(),
			PackageState::config_only(Some("1.0".to_string()), Some(false)),
		)
	}

	#[test]
	fn sorted_summary_sets_use_transaction_order_and_skip_empty_sets() {
		let mut pkg_set = HashMap::new();
		pkg_set.insert(
			Operation::Upgrade,
			vec![transition("upgrade", Operation::Upgrade)],
		);
		pkg_set.insert(
			Operation::Remove,
			vec![transition("remove", Operation::Remove)],
		);
		pkg_set.insert(Operation::Install, Vec::new());
		pkg_set.insert(Operation::Held, vec![transition("held", Operation::Held)]);

		let operations = sorted_summary_sets(&pkg_set)
			.into_iter()
			.map(|(operation, _)| operation)
			.collect::<Vec<_>>();

		assert_eq!(
			operations,
			vec![Operation::Remove, Operation::Upgrade, Operation::Held]
		);
	}
}
