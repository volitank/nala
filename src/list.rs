use std::io::Write;

use anyhow::{anyhow, Result};
use rust_apt::cache::{Cache, PackageSort};
use rust_apt::package::{Package, Version};

use crate::colors::Color;
use crate::config::Config;

/// The list command
pub fn list(config: &Config, color: &Color) -> Result<()> {
	let stdout = std::io::stdout();
	let mut out = stdout.lock();

	let cache = Cache::new();
	let mut sort = PackageSort::default().names();

	// set up our sorting parameters
	if config.get_bool("installed", false) {
		sort = sort.installed();
	}

	if config.get_bool("upgradable", false) {
		sort = sort.upgradable();
	}

	if config.get_bool("virtual", false) {
		sort = sort.only_virtual();
	}

	// Filter the packages by names if they were provided
	let mut packages: Vec<Package> = Vec::new();
	let mut not_found: Vec<&String> = Vec::new();
	match config.pkg_names() {
		Some(pkg_names) => {
			// Get only the packages of the name
			for name in pkg_names {
				if let Some(pkg) = cache.get(name) {
					packages.push(pkg);
					continue;
				}
				// Keep track of names that were not found
				not_found.push(name)
			}
			// Sort the packages by name so it's pretty
			packages.sort_by_cached_key(|pkg| pkg.fullname(true));
		},
		None => {
			// No package were specified by the user so get them all.
			packages = cache.packages(&sort).collect::<Vec<Package>>();
		},
	}

	// If packages are empty then there is nothing to list.
	if packages.is_empty() {
		return Err(anyhow!("Nothing was found to list"));
	}

	// We at least have one package so we can begin listing.
	for pkg in packages {
		if config.get_bool("all_versions", false) && pkg.has_versions() {
			for version in pkg.versions() {
				write!(out, "{} ", color.package(&pkg.fullname(true)))?;
				list_version(&mut out, color, &pkg, &version)?;
				list_description(&mut out, config, &version)?;
			}
			// The new line is a little weird if we print descriptions
			if !config.get_bool("description", false) && !config.get_bool("summary", false) {
				// New line to separate package groups
				writeln!(out)?;
			}
			continue;
		}

		// Write the package name
		write!(out, "{} ", color.package(&pkg.fullname(true)))?;

		// The first version in the list should be the latest
		if let Some(version) = pkg.versions().next() {
			// There is a version! Let's format it
			list_version(&mut out, color, &pkg, &version)?;
			list_description(&mut out, config, &version)?;
			continue;
		}

		// There are no versions so it must be a virtual package
		list_virtual(&mut out, color, &cache, &pkg)?;
	}

	// Alert the user of any patterns that were not found
	for name in not_found {
		color.warn(&format!("'{name}' was not found"));
	}

	Ok(())
}

/// List a single version of a package
fn list_version(
	out: &mut impl std::io::Write,
	color: &Color,
	pkg: &Package,
	version: &Version,
) -> std::io::Result<()> {
	// Add the version to the string
	write!(out, "{}", color.version(&version.version()))?;

	if let Some(pkg_file) = version.package_files().next() {
		let archive = pkg_file
			.archive()
			.unwrap_or_else(|| String::from("Unknown"));

		if archive != "now" {
			let origin = pkg_file.origin().unwrap_or_else(|| String::from("Unknown"));
			let component = pkg_file
				.component()
				.unwrap_or_else(|| String::from("Unknown"));
			write!(out, " [{origin}/{archive} {component}]")?;
			// write!(out, " [local]")?;
			// Do we want to show something for this? Kind of handled elsewhere
		}
	}

	// There is both an installed and candidate version
	if let (Some(installed), Some(candidate)) = (pkg.installed(), pkg.candidate()) {
		// Version is installed, check if it's upgradable
		if version == &installed && version < &candidate {
			return writeln!(
				out,
				" [Installed, Upgradable to: {}]",
				color.version(&candidate.version()),
			);
		}
		// Version isn't installed, see if it's the candidate
		if version == &candidate && version > &installed {
			return writeln!(
				out,
				" [Upgradable from: {}]",
				color.version(&installed.version()),
			);
		}
	}

	// The version will not have an upgradable string, but is installed
	if version.is_installed() {
		// Version isn't downloadable, consider it locally installed
		if !version.downloadable() {
			return writeln!(out, " [Installed, Local]");
		}

		if pkg.is_auto_removable() {
			return writeln!(out, " [Installed, Auto-Removable]");
		}

		if pkg.is_auto_installed() {
			return writeln!(out, " [Installed, Automatic]");
		}

		// None of the installed conditions were met
		return writeln!(out, " [Installed]");
	}

	// Conditions aren't met, return the package name and version
	out.write_all(b"\n")
}

/// List the description or summary if requested
fn list_description(
	out: &mut impl std::io::Write,
	config: &Config,
	version: &Version,
) -> Result<()> {
	if config.get_bool("description", false) {
		writeln!(out, " {}\n", version.description())?;
	}
	if config.get_bool("summary", false) {
		writeln!(out, " {}\n", version.summary())?;
	}
	Ok(())
}

fn list_virtual(
	out: &mut impl std::io::Write,
	color: &Color,
	cache: &Cache,
	pkg: &Package,
) -> Result<()> {
	// There are no versions so it must be a virtual package
	write!(
		out,
		"{}{}{} ",
		color.bold("("),
		color.yellow("Virtual Package"),
		color.bold(")")
	)?;

	let provides = cache
		.provides(pkg, true)
		.map(|p| p.fullname(true))
		.collect::<Vec<String>>();

	// If the virtual package provides anything we can show it
	if !provides.is_empty() {
		writeln!(
			out,
			"\n  {} {}",
			color.bold("Provides:"),
			&provides.join(", "),
		)?;
	} else {
		writeln!(out, "\n  Nothing provides this package.")?;
	}
	Ok(())
}
