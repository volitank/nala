use anyhow::{bail, Result};
use glob::Pattern;
use regex::{Regex, RegexBuilder};
use rust_apt::cache::{Cache, PackageSort};
use rust_apt::package::{Package, Version};

use crate::colors::Color;
use crate::config::Config;

/// Turn an iterator of strings into glob patters.
pub fn get_regexes<T: AsRef<str>>(strings: &[T]) -> Result<Vec<Regex>> {
	let mut regex = Vec::new();
	for s in strings {
		regex.push(
			RegexBuilder::new(s.as_ref())
				.case_insensitive(true)
				.build()?,
		);
	}
	Ok(regex)
}

/// Turn an iterator of strings into glob patters.
pub fn get_globs<T: AsRef<str>>(strings: &[T]) -> Result<Vec<Pattern>> {
	let mut globs = Vec::new();
	for string in strings {
		globs.push(Pattern::new(string.as_ref())?);
	}
	Ok(globs)
}

/// The search command
pub fn search(config: &Config, color: &Color) -> Result<()> {
	let stdout = std::io::stdout();
	let mut out = stdout.lock();

	let cache = Cache::new();
	let sort = get_sorter(config);

	// Eventually maybe allow more than one option to search with
	let patterns = match config.pkg_names() {
		Some(vec) => get_regexes(vec)?,
		None => bail!("You must give at least one search Pattern"),
	};
	let all_pkgs = cache.packages(&sort).collect::<Vec<Package>>();
	let mut packages: Vec<&Package> = Vec::new();
	let mut not_found: Vec<String> = Vec::new();
	for regex in &patterns {
		let mut found = false;
		// Match the packages based on the regex
		for pkg in &all_pkgs {
			// If the name matches we can stop here
			if regex.is_match(&pkg.name()) {
				found = true;
				packages.push(pkg);
				continue;
			}
			// Check all of the versions source name and description for a match
			// Getting a description has a performance penalty due to a records lookup
			// Because of this it is last in the chain
			for ver in pkg.versions() {
				if regex.is_match(&ver.source_name()) || regex.is_match(&ver.description()) {
					found = true;
					packages.push(pkg);
					continue;
				}
			}
		}
		// Keep track of names that were not found
		if !found {
			not_found.push(regex.as_str().to_owned())
		}
	}

	// List the packages that were found
	list_packages(packages, &cache, config, color, &mut out)?;

	// Alert the user of any patterns that were not found
	for name in not_found {
		color.warn(&format!("'{name}' was not found"));
	}

	Ok(())
}

/// The list command
pub fn list(config: &Config, color: &Color) -> Result<()> {
	let stdout = std::io::stdout();
	let mut out = stdout.lock();

	let cache = Cache::new();
	let sort = get_sorter(config);

	// Filter the packages by names if they were provided
	let mut packages: Vec<&Package> = Vec::new();
	let mut not_found: Vec<String> = Vec::new();
	let all_pkgs = cache.packages(&sort).collect::<Vec<Package>>();
	match config.pkg_names() {
		Some(pkg_names) => {
			// Get our package matches from the specified glob
			for glob in &get_globs(pkg_names)? {
				let mut found = false;
				for pkg in &all_pkgs {
					if glob.matches(&pkg.name()) {
						found = true;
						packages.push(pkg);
					}
				}
				// Keep track of names that were not found
				if !found {
					not_found.push(glob.as_str().to_owned())
				}
			}
		},
		None => {
			// No package were specified by the user so get them all
			packages = all_pkgs.iter().collect();
		},
	}

	// List the packages that were found
	list_packages(packages, &cache, config, color, &mut out)?;

	// Alert the user of any patterns that were not found
	for name in not_found {
		color.warn(&format!("'{name}' was not found"));
	}

	Ok(())
}

/// List packages in a vector
///
/// Shared function between list and search
fn list_packages(
	packages: Vec<&Package>,
	cache: &Cache,
	config: &Config,
	color: &Color,
	out: &mut impl std::io::Write,
) -> Result<()> {
	// If packages are empty then there is nothing to list.
	if packages.is_empty() {
		bail!("Nothing was found to list");
	}

	// We at least have one package so we can begin listing.
	for pkg in packages {
		if config.get_bool("all_versions", false) && pkg.has_versions() {
			for version in pkg.versions() {
				write!(out, "{} ", color.package(&pkg.fullname(true)))?;
				list_version(out, color, pkg, &version)?;
				list_description(out, config, &version)?;
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
			list_version(out, color, pkg, &version)?;
			list_description(out, config, &version)?;
			continue;
		}

		// There are no versions so it must be a virtual package
		list_virtual(out, color, cache, pkg)?;
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

/// List a virtual package
fn list_virtual(
	out: &mut impl std::io::Write,
	color: &Color,
	cache: &Cache,
	pkg: &Package,
) -> Result<()> {
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
			color.bold("Provided By:"),
			&provides.join(", "),
		)?;
	} else {
		writeln!(out, "\n  Nothing provides this package.")?;
	}
	Ok(())
}

/// Configure sorter for list and search
fn get_sorter(config: &Config) -> PackageSort {
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
	sort
}

#[cfg(test)]
mod test {
	use crate::list::*;

	#[test]
	fn virtual_list() {
		let mut out = Vec::new();
		let cache = Cache::new();
		let color = Color::default();

		// Package selection is based on current Debian Sid
		// These tests may not be consistent across distributions
		let pkg = cache.get("matemenu").unwrap();
		list_virtual(&mut out, &color, &cache, &pkg).unwrap();
		// Just a print so the output looks better in the tests
		println!("\n");

		// Convert the vector of bytes to a string
		let output = std::str::from_utf8(&out).unwrap();

		// Set up what the common string between the two tests are
		let mut virt = String::from("\u{1b}[1m(\u{1b}[0m\u{1b}[1;38;5;11m");
		virt += "Virtual Package";
		virt += "\u{1b}[0m\u{1b}[1m)\u{1b}[0m \n";

		// Create the specific string for a virtual package that provides nothing
		let mut string = virt.clone();
		string += "  Nothing provides this package.\n";

		// Test if we're correct or not
		dbg!(&output);
		dbg!(&string);
		assert_eq!(output, string);

		let mut out = Vec::new();
		let pkg = cache.get("systemd-sysusers").unwrap();

		// Do the same thing again but with a virtual package that provides
		list_virtual(&mut out, &color, &cache, &pkg).unwrap();

		// Set up what the correct output should be
		let mut string = virt.clone();
		string += "  ";
		string += &color.bold("Provided By:");
		string += " systemd-standalone-sysusers, systemd, opensysusers\n";

		// Convert the vector of bytes to a string
		let output = std::str::from_utf8(&out).unwrap();

		dbg!(&output);
		dbg!(&string);
		assert_eq!(output, string);
	}

	#[test]
	fn description() {
		let mut out = Vec::new();
		let cache = Cache::new();
		let mut config = Config::default();

		// Set the description to true so that we are able to get it
		config.set_bool("description", true);

		let pkg = cache.get("dpkg").unwrap();

		list_description(&mut out, &config, &pkg.candidate().unwrap()).unwrap();

		// Convert the vector of bytes to a string
		let output = std::str::from_utf8(&out).unwrap();

		// Match the description. This may change with different versions of dpkg
		let string = " Debian package management system\n This package provides the low-level \
		              infrastructure for handling the\n installation and removal of Debian \
		              software packages.\n .\n For Debian package development tools, install \
		              dpkg-dev.\n\n";

		dbg!(&output);
		dbg!(string);
		assert_eq!(output, string);

		// Reset and change the environment for the summary test
		let mut out = Vec::new();

		config.set_bool("description", false);
		config.set_bool("summary", true);

		list_description(&mut out, &config, &pkg.candidate().unwrap()).unwrap();

		// Convert the vector of bytes to a string
		let output = std::str::from_utf8(&out).unwrap();

		// Match the summary. This may change with different versions of dpkg
		let string = " Debian package management system\n\n";

		dbg!(&output);
		dbg!(&string);
		assert_eq!(output, string);
	}

	#[test]
	fn version() {
		let mut out = Vec::new();
		let cache = Cache::new();
		let color = Color::default();

		let pkg = cache.get("dpkg").unwrap();

		list_version(&mut out, &color, &pkg, &pkg.candidate().unwrap()).unwrap();

		// Convert the vector of bytes to a string
		let output = std::str::from_utf8(&out).unwrap();

		// Match the description. This may change with different versions of dpkg
		let mut string = String::from("\u{1b}[1m(\u{1b}[0m\u{1b}[1;38;5;12m");
		string += "1.21.9";
		string += "\u{1b}[0m\u{1b}[1m)\u{1b}[0m [Debian/unstable main] [Installed]\n";

		dbg!(&output);
		dbg!(&string);
		assert_eq!(output, string);
	}

	#[test]
	fn glob() {
		let cache = Cache::new();
		let sort = PackageSort::default().names();
		let all_pkgs = cache.packages(&sort).collect::<Vec<Package>>();
		let mut packages = std::collections::HashSet::new();

		// Results are based on Debian Sid
		// These results could change and require updating
		for glob in &get_globs(&["apt?y", "aptly*"]).unwrap() {
			for pkg in &all_pkgs {
				let pkg_name = pkg.name();
				if glob.matches(&pkg_name) {
					packages.insert(pkg_name);
				}
			}
		}

		// print just for easy debugging later
		for pkg_name in &packages {
			println!("{pkg_name}")
		}
		// Currently there are 3 package names that should match
		assert_eq!(packages.len(), 3);
	}

	#[test]
	fn regex() {
		let cache = Cache::new();
		let sort = PackageSort::default().names();
		let all_pkgs = cache.packages(&sort).collect::<Vec<Package>>();
		let mut packages = std::collections::HashSet::new();

		// This regex should pull in only dpkg and apt
		for regex in &get_regexes(&[r"^dpk.$", r"^apt$"]).unwrap() {
			// Match the packages based on the regex
			for pkg in &all_pkgs {
				let pkg_name = pkg.name();
				// Add matches to our set
				if regex.is_match(&pkg_name) {
					packages.insert(pkg_name);
				}
			}
		}

		// print just for easy debugging later
		for pkg_name in &packages {
			println!("{pkg_name}")
		}
		// Should only contain 2 packages, dpkg and apt
		assert_eq!(packages.len(), 2);
	}
}
