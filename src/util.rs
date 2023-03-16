#[macro_export]
/// Print Debug information if the option is set
macro_rules! dprint {
	($config:expr $(,)?, $($arg: tt)*) => {
		if $config.debug() {
			let string = std::fmt::format(std::format_args!($($arg)*));
			eprintln!("DEBUG: {string}");
		}
	};
}
use std::collections::HashSet;

use anyhow::{bail, Result};
pub use dprint;
use globset::GlobBuilder;
use regex::{Regex, RegexBuilder};
use rust_apt::cache::Cache;
use rust_apt::package::{Package, Version};

use crate::config::Config;

pub struct Matcher {
	regexs: Vec<Regex>,
}

impl Matcher {
	/// Simple wrapper to easy create regex only
	pub fn new_regex(regexs: Vec<Regex>) -> Matcher { Matcher { regexs } }

	/// Turn an iterator of strings into regex patterns.
	pub fn from_regexs<T: AsRef<str>>(strings: &[T]) -> Result<Matcher> {
		let mut regex = Vec::new();
		for string in strings {
			regex.push(
				RegexBuilder::new(string.as_ref())
					.case_insensitive(true)
					.build()?,
			);
		}
		Ok(Matcher::new_regex(regex))
	}

	/// Matches only package names.
	/// Return found Packages, and not found regex &str.
	///
	/// names_only = true will match only against pkg names.
	pub fn regex_pkgs<'a, Container: IntoIterator<Item = Package<'a>>>(
		&self,
		packages: Container,
		names_only: bool,
	) -> (Vec<Package<'a>>, HashSet<String>) {
		let mut found_pkgs = Vec::new();
		let mut not_found =
			HashSet::from_iter(self.regexs.iter().map(|regex| regex.as_str().to_string()));

		'outer: for pkg in packages {
			// Check for pkg name matches first.
			for regex in &self.regexs {
				if regex.is_match(pkg.name()) {
					found_pkgs.push(pkg);
					not_found.remove(regex.as_str());
					// Continue with packages as we don't want to hit versions if we can help it.
					continue 'outer;
				}
			}

			// If we only want names we can skip the descriptions
			if names_only {
				continue;
			}

			// Search all versions for a matching description
			for ver in pkg.versions().collect::<Vec<Version>>() {
				if let Some(desc) = ver.description() {
					for regex in &self.regexs {
						if regex.is_match(&desc) {
							found_pkgs.push(pkg);
							not_found.remove(regex.as_str());
							continue 'outer;
						}
					}
				}
			}
		}
		(found_pkgs, not_found)
	}
}

pub fn glob_pkgs<'a, Container: IntoIterator<Item = Package<'a>>, T: AsRef<str>>(
	glob_strings: &[T],
	packages: Container,
) -> Result<(Vec<Package<'a>>, HashSet<String>)> {
	let mut found_pkgs = Vec::new();

	// Build the glob patterns from the strings provided
	let mut globs = vec![];
	for string in glob_strings {
		globs.push(
			GlobBuilder::new(string.as_ref())
				.case_insensitive(true)
				.build()?
				.compile_matcher(),
		)
	}

	let mut not_found = HashSet::from_iter(globs.iter().map(|glob| glob.glob().to_string()));

	for pkg in packages {
		// Check for pkg name matches first.
		for glob in &globs {
			if glob.is_match(pkg.fullname(true)) {
				found_pkgs.push(pkg);
				// Globble Globble Globble this gives us a &str lol
				not_found.remove(glob.glob().glob());
				// We have already moved the package so we need to just continue
				break;
			}
		}
	}
	Ok((found_pkgs, not_found))
}

pub fn virtual_filter<'a, Container: IntoIterator<Item = Package<'a>>>(
	packages: Container,
	cache: &'a Cache,
	config: &Config,
) -> Result<HashSet<Package<'a>>> {
	let mut virtual_filtered = HashSet::new();
	for pkg in packages {
		// If the package has versions then it isn't virtual
		// just push it and continue
		if pkg.has_versions() {
			virtual_filtered.insert(pkg);
			continue;
		}

		// If the package doesn't have provides it's purely virtual
		// There is nothing that can satisfy it. Referenced only by name
		// At time of commit `python3-libmapper` is purely virtual
		if !pkg.has_provides() {
			config.color.warn(&format!(
				"{} has no providers and is purely virutal",
				config.color.package(pkg.name())
			));
			continue;
		}

		// Package is virtual so get its providers.
		// HashSet for duplicated packages when there is more than one version
		let providers: HashSet<Package> = pkg.provides().map(|p| p.package()).collect();

		// If there is only one provider just select that as the target
		if providers.len() == 1 {
			// Unwrap should be fine here, we know that there is 1 in the Vector.
			let target = providers.into_iter().next().unwrap();
			config.color.notice(&format!(
				"Selecting {} instead of virtual package {}",
				config.color.package(&target.fullname(false)),
				config.color.package(pkg.name())
			));

			// Unwrap should be fine here because we know the name.
			// We have to grab the package from the cache again because
			// Provider lifetimes are a bit goofy.
			virtual_filtered.insert(cache.get(&target.fullname(false)).unwrap());
			continue;
		}

		// If there are multiple providers then we will error out
		// and show the packages the user could select instead.
		if providers.len() > 1 {
			println!(
				"{} is a virtual package provided by:",
				config.color.package(pkg.name())
			);
			for target in &providers {
				// If the version doesn't have a candidate no sense in showing it
				if let Some(cand) = target.candidate() {
					println!(
						"    {} {}",
						config.color.package(&target.fullname(true)),
						config.color.version(cand.version()),
					)
				}
			}
			bail!("You should select just one.")
		}
	}
	Ok(virtual_filtered)
}
