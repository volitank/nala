use std::collections::btree_map::Entry;
use std::collections::{BTreeMap, BTreeSet};

use anyhow::{bail, Result};
use globset::GlobBuilder;
use regex::{Regex, RegexBuilder};
use rust_apt::raw::IntoRawIter;
use rust_apt::{Cache, Package, PackageSort, Version};

use crate::cmd::Operation;
use crate::config::{color, Config, Theme};
use crate::libnala::NalaPkg;
use crate::{debug, error, info};

#[derive(Debug, Default)]
pub struct Selection<'a> {
	resolved: Vec<ResolvedPkg<'a>>,
	missing: Vec<String>,
}

#[derive(Debug)]
pub struct ResolvedPkg<'a> {
	pub pkg: Package<'a>,
	pub modifier: Option<Operation>,
	pub candidate: Option<Version<'a>>,
}

impl<'a> Selection<'a> {
	pub fn add(
		&mut self,
		pkg: Package<'a>,
		candidate: Option<Version<'a>>,
		modifier: Option<Operation>,
	) {
		self.resolved.push(ResolvedPkg {
			pkg,
			modifier,
			candidate,
		});
	}

	pub fn check_not_found(&self) -> Result<()> {
		if self.missing.is_empty() {
			return Ok(());
		}

		debug!("{:#?}", self.missing);
		for missing in &self.missing {
			error!("'{}' was not found", color::color!(Theme::Notice, missing));
		}

		bail!("Some packages were not found in the cache")
	}

	pub fn into_packages_and_missing(self) -> (Vec<Package<'a>>, Vec<String>) {
		let mut seen = BTreeSet::<(String, String)>::new();
		let mut pkgs = Vec::new();

		for found in self.resolved {
			let key = (found.pkg.name().to_string(), found.pkg.arch().to_string());
			if seen.insert(key) {
				pkgs.push(found.pkg);
			}
		}

		(pkgs, self.missing)
	}

	pub fn mark(self, cache: &Cache, default_op: Operation, purge: bool) -> Result<()> {
		self.check_not_found()?;
		let _ = unsafe { cache.depcache().action_group() };
		let mut merged = BTreeMap::<(String, String), ResolvedPkg<'a>>::new();

		for item in self.resolved {
			let key = (item.pkg.name().to_string(), item.pkg.arch().to_string());
			match merged.entry(key) {
				Entry::Vacant(entry) => {
					entry.insert(item);
				},
				Entry::Occupied(mut entry) => {
					let existing = entry.get_mut();
					let existing_op = existing.modifier.unwrap_or(default_op);
					let new_op = item.modifier.unwrap_or(default_op);
					if existing_op != new_op {
						bail!(
							"Conflicting operations for '{}': {} vs {}",
							item.pkg.name(),
							existing_op,
							new_op
						);
					}

					if let (Some(existing_candidate), Some(new_candidate)) =
						(&existing.candidate, &item.candidate)
					{
						if existing_candidate.version() != new_candidate.version() {
							bail!(
								"Conflicting pinned versions for '{}': {} vs {}",
								item.pkg.name(),
								existing_candidate.version(),
								new_candidate.version()
							);
						}
					} else if existing.candidate.is_none() && item.candidate.is_some() {
						existing.candidate = item.candidate;
					}
				},
			}
		}

		for item in merged.into_values() {
			let pkg = &item.pkg;
			let op = item.modifier.unwrap_or(default_op);

			if op == Operation::Install {
				if let Some(candidate) = item.candidate {
					candidate.set_candidate();
				}
			}

			match op {
				Operation::Install => {
					let Some(cand) = pkg.candidate() else {
						bail!("{} has no install candidate", pkg.name())
					};

					if let Some(inst) = pkg.installed() {
						if inst == cand {
							info!(
								"{}{} is already installed and at the latest version",
								color::primary!(pkg.name()),
								color::ver!(cand.version())
							);
							continue;
						}
					}

					cache.resolver().clear(pkg);
					cache.resolver().protect(pkg);
					pkg.mark_install(true, true);
				},
				Operation::Remove => {
					let Some(_inst) = pkg.installed() else {
						info!("{} is not installed", pkg.name());
						continue;
					};

					debug!("Mark Delete: {pkg}");
					cache.resolver().clear(pkg);
					cache.resolver().protect(pkg);
					pkg.mark_delete(purge);
				},
				_ => todo!(),
			}
		}

		Ok(())
	}
}

pub fn log_missing_notices(missing: &[String]) {
	for token in missing {
		info!("'{}' was not found", color::color!(Theme::Notice, token));
	}
}

fn parse_trailing_modifier(raw: &str) -> (&str, Option<Operation>) {
	if let Some(without) = raw.strip_suffix('+') {
		(without, Some(Operation::Install))
	} else if let Some(without) = raw.strip_suffix('-') {
		(without, Some(Operation::Remove))
	} else {
		(raw, None)
	}
}

fn contains_glob_metachar(pattern: &str) -> bool {
	pattern
		.chars()
		.any(|c| matches!(c, '*' | '?' | '[' | ']' | '{' | '}'))
}

fn build_glob_matcher(pattern: &str) -> Result<globset::GlobMatcher> {
	Ok(GlobBuilder::new(pattern)
		.case_insensitive(true)
		.build()?
		.compile_matcher())
}

fn parse_version_pin(raw: &str) -> Result<(String, String, Option<Operation>)> {
	let (without_modifier, modifier) = parse_trailing_modifier(raw);
	let Some((name, version)) = without_modifier.split_once('=') else {
		bail!("Invalid version pin: '{raw}'");
	};

	if name.is_empty() || version.is_empty() {
		bail!("Invalid version pin: '{raw}'");
	}

	if contains_glob_metachar(name) {
		bail!("Version pin requires an exact package name: '{raw}'");
	}

	Ok((name.to_string(), version.to_string(), modifier))
}

fn arch_matches(arch: &str, arches: &[String]) -> bool {
	arch == "all" || arches.iter().any(|a| a == arch)
}

fn find_matching_pkgs<'a>(
	cache: &'a Cache,
	config: &Config,
	matcher: &globset::GlobMatcher,
	arches: &[String],
) -> Vec<Package<'a>> {
	cache
		.packages(&get_sorter(config))
		.filter(|pkg| arch_matches(pkg.arch(), arches))
		.filter(|pkg| matcher.is_match(pkg.name()))
		.collect::<Vec<_>>()
}

pub fn get_sorter(config: &Config) -> PackageSort {
	let mut sort = PackageSort::default().include_virtual();

	if config.get_bool("installed", false) {
		sort = sort.installed();
	}

	if config.get_bool("upgradable", false) {
		sort = sort.upgradable();
	}

	sort
}

pub fn pkgs_with_modifiers<'a>(
	cli_pkgs: Vec<String>,
	config: &Config,
	cache: &'a Cache,
) -> Result<Selection<'a>> {
	let mut selection = Selection::default();
	let arches = config.arches();

	for raw in cli_pkgs {
		if raw.contains('=') {
			let (name, version, modifier) = parse_version_pin(&raw)?;
			let name_matcher = build_glob_matcher(&name)?;
			let pkgs = find_matching_pkgs(cache, config, &name_matcher, &arches);
			if pkgs.is_empty() {
				selection.missing.push(raw);
				continue;
			}

			for pkg in pkgs {
				let pkg = pkg.filter_virtual()?;
				let Some(ver) = pkg.get_version(&version) else {
					bail!("Unable to find version '{}' for '{}'", version, pkg.name());
				};
				selection.add(pkg, Some(ver), modifier);
			}
			continue;
		}

		let raw_matcher = build_glob_matcher(&raw)?;
		let raw_matches = find_matching_pkgs(cache, config, &raw_matcher, &arches);
		if !raw_matches.is_empty() {
			for pkg in raw_matches {
				let pkg = pkg.filter_virtual()?;
				selection.add(pkg, None, None);
			}
			continue;
		}

		let (fallback_pattern, modifier) = match parse_trailing_modifier(&raw) {
			(fallback, Some(modifier)) => (fallback, modifier),
			(_, None) => {
				selection.missing.push(raw);
				continue;
			},
		};

		if fallback_pattern.is_empty() {
			bail!("Invalid package name: '{raw}'");
		}

		let fallback_matcher = build_glob_matcher(fallback_pattern)?;
		let fallback_matches = find_matching_pkgs(cache, config, &fallback_matcher, &arches);
		if fallback_matches.is_empty() {
			selection.missing.push(raw);
			continue;
		}

		for pkg in fallback_matches {
			let pkg = pkg.filter_virtual()?;
			selection.add(pkg, None, Some(modifier));
		}
	}

	Ok(selection)
}

pub fn regex_pkgs<'a>(config: &Config, cache: &'a Cache) -> Result<Vec<Package<'a>>> {
	let patterns = config
		.pkg_names()?
		.into_iter()
		.map(|pattern| RegexBuilder::new(&pattern).case_insensitive(true).build())
		.collect::<Result<Vec<Regex>, _>>()?;

	let names_only = config.get_bool("names_only", false);
	let arches = config.apt.get_architectures();
	let primary_arch = arches.first().map(String::as_str).unwrap_or("all");

	let mut seen = BTreeSet::<(String, String)>::new();
	let mut matches = Vec::new();

	for pkg in cache.packages(&get_sorter(config)) {
		if pkg.arch() != "all" && pkg.arch() != primary_arch {
			continue;
		}

		if patterns.iter().any(|re| re.is_match(pkg.name())) {
			let key = (pkg.name().to_string(), pkg.arch().to_string());
			if seen.insert(key) {
				matches.push(pkg);
			}
			continue;
		}

		if names_only {
			continue;
		}

		let Some(version) = pkg.versions().next() else {
			continue;
		};
		let desc = unsafe { version.translated_desc().make_safe() };
		let Some(desc) = desc.and_then(|d| cache.records().desc_lookup(&d).long_desc()) else {
			continue;
		};

		if patterns.iter().any(|re| re.is_match(&desc)) {
			let key = (pkg.name().to_string(), pkg.arch().to_string());
			if seen.insert(key) {
				matches.push(pkg);
			}
		}
	}

	Ok(matches)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn version_pin_requires_exact_name() {
		let err = parse_version_pin("acl*=2.3.2-2").unwrap_err();
		assert!(err
			.to_string()
			.contains("Version pin requires an exact package name"));
	}

	#[test]
	fn version_pin_rejects_empty_name_or_version() {
		let err = parse_version_pin("=1.2.3").unwrap_err();
		assert!(err.to_string().contains("Invalid version pin"));

		let err = parse_version_pin("acl=").unwrap_err();
		assert!(err.to_string().contains("Invalid version pin"));
	}

	#[test]
	fn glob_token_rejects_empty_fallback_pattern() {
		let (fallback, modifier) = parse_trailing_modifier("+");
		assert!(fallback.is_empty());
		assert_eq!(modifier, Some(Operation::Install));
	}

	#[test]
	fn glob_token_fallback_is_stripped_pattern() {
		let (fallback, modifier) = parse_trailing_modifier("foo+");
		assert_eq!(fallback, "foo");
		assert_eq!(modifier, Some(Operation::Install));
	}
}
