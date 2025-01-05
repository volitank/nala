use std::marker::PhantomData;

use anyhow::{bail, Result};
use globset::{GlobBuilder, GlobMatcher};
use regex::{Regex, RegexBuilder};
use rust_apt::raw::IntoRawIter;
use rust_apt::{Cache, Package, PackageSort, Version};

use crate::cmd::Operation;
use crate::config::{color, Config, Theme};
use crate::libnala::NalaPkg;
use crate::{debug, error, info};

#[derive(Debug)]
pub enum Matcher {
	Glob(GlobMatcher),
	Regex(Regex),
}

#[derive(Debug)]
pub struct CliPackages<'a>(Vec<CliPackage<'a>>);

#[derive(Debug)]
// TODO: Maybe we want to be able to skip adding matcher for install command?
pub struct CliPackage<'a> {
	pub name: String,
	version: Option<String>,
	pub modifier: Option<Operation>,
	matcher: Matcher,
	pub pkgs: Vec<FoundPackage<'a>>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct FoundPackage<'a> {
	pub pkg: Package<'a>,
	// I suspect we will want the selected version at some point
	pub _version: Version<'a>,
	pub modifier: Option<Operation>,
	_marker: PhantomData<&'a Cache>,
}

impl<'a> FromIterator<CliPackage<'a>> for CliPackages<'a> {
	fn from_iter<I: IntoIterator<Item = CliPackage<'a>>>(iter: I) -> Self {
		CliPackages(iter.into_iter().collect())
	}
}

impl<'a> CliPackages<'a> {
	pub fn new() -> CliPackages<'a> { Self(vec![]) }

	pub fn find_mut(&mut self, haystack: &str) -> Option<&mut CliPackage<'a>> {
		self.0.iter_mut().find(|cli| cli.is_match(haystack))
	}

	pub fn push(&mut self, cli: CliPackage<'a>) { self.0.push(cli) }

	pub fn sort_by_name(&mut self) {
		self.0
			.sort_by_cached_key(|c| c.pkgs.first().unwrap().pkg.name().to_string());
	}

	pub fn only_pkgs(self) -> Vec<Package<'a>> { self.found().map(|p| p.pkg).collect() }

	/// Consume the iterator and retrieve all the pkgs found
	pub fn found(self) -> impl Iterator<Item = FoundPackage<'a>> {
		self.0.into_iter().flat_map(|p| p.pkgs)
	}

	pub fn found_as_ref(&self) -> Vec<&FoundPackage<'a>> {
		self.iter().flat_map(|p| &p.pkgs).collect()
	}

	pub fn iter(&self) -> impl Iterator<Item = &CliPackage<'a>> { self.0.iter() }

	pub fn check_not_found(&self) -> Result<()> {
		debug!("{:#?}", self.found_as_ref());
		let mut bail = false;
		for cli in &self.0 {
			if !cli.pkgs.is_empty() {
				continue;
			}
			error!(
				"'{}' was not found",
				color::color!(Theme::Notice, &cli.name)
			);
			bail = true;
		}

		if bail {
			bail!("Some packages were not found in the cache")
		}

		Ok(())
	}

	pub fn mark(self, cache: &Cache, operation: Operation, purge: bool) -> Result<()> {
		let _ = unsafe { cache.depcache().action_group() };
		for found in self.found() {
			let pkg = &found.pkg;
			match found.modifier.unwrap_or(operation) {
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

					// TODO: Apt has this, I think we need to bind this in rust-apt though
					// Potentially can call it pkg.mark_hold()?
					//
					// MarkInstall refuses to install packages on hold
					// Pkg->SelectedState = pkgCache::State::Hold;
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

impl<'a> CliPackage<'a> {
	pub fn new_glob(name: String) -> Result<Self> {
		let matcher = Matcher::Glob(
			GlobBuilder::new(&name)
				.case_insensitive(true)
				.build()?
				.compile_matcher(),
		);
		Ok(Self::new(name, matcher))
	}

	pub fn new_regex(name: String) -> Result<Self> {
		let matcher = Matcher::Regex(RegexBuilder::new(&name).case_insensitive(true).build()?);
		Ok(Self::new(name, matcher))
	}

	pub fn new(name: String, matcher: Matcher) -> CliPackage<'a> {
		Self {
			name,
			version: None,
			modifier: None,
			matcher,
			pkgs: vec![],
		}
	}

	pub fn modifier(mut self, value: Option<Operation>) -> Self {
		self.modifier = value;
		self
	}

	pub fn with_pkg(mut self, pkg: Package<'a>, ver: Version<'a>) -> Self {
		self.add_no_op(pkg, ver);
		self
	}

	pub fn add_pkg(&mut self, pkg: Package<'a>, ver: Version<'a>, op: Option<Operation>) {
		self.pkgs.push(FoundPackage::new(pkg, ver, op))
	}

	pub fn add_no_op(&mut self, pkg: Package<'a>, version: Version<'a>) {
		self.add_pkg(pkg, version, None);
	}

	pub fn set_ver(&mut self, ver_str: String) { self.version = Some(ver_str); }

	pub fn get_version(&self, pkg: &Package<'a>) -> Result<Version<'a>> {
		if let Some(ver_str) = &self.version {
			if let Some(ver) = pkg.get_version(ver_str) {
				return Ok(ver);
			}
			bail!("Unable to find version '{ver_str}' for '{}'", pkg.name());
		};

		if let Some(ver) = pkg.versions().next() {
			return Ok(ver);
		}

		bail!("Unable to find any versions for '{}'", pkg.name());
	}

	pub fn is_match(&self, other: &str) -> bool {
		match &self.matcher {
			Matcher::Glob(glob) => glob.is_match(other),
			Matcher::Regex(regex) => regex.is_match(other),
		}
	}
}

impl<'a> FoundPackage<'a> {
	pub fn new(
		pkg: Package<'a>,
		_version: Version<'a>,
		modifier: Option<Operation>,
	) -> FoundPackage<'a> {
		Self {
			pkg,
			_version,
			modifier,
			_marker: PhantomData,
		}
	}
}

fn split_version(cli_str: String) -> (String, Option<String>) {
	if let Some(split) = cli_str.split_once("=") {
		(split.0.to_string(), Some(split.1.to_string()))
	} else {
		(cli_str, None)
	}
}

pub fn get_sorter(config: &Config) -> PackageSort {
	// Configure sorter for list and search
	let mut sort = PackageSort::default().include_virtual();

	// set up our sorting parameters
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
) -> Result<CliPackages<'a>> {
	debug!("Start Globbing cli_pkgs {cli_pkgs:#?}");
	let mut globs = CliPackages::new();
	for mut pkg in cli_pkgs {
		let mut modifier = None;
		for (modi, op) in [("-", Operation::Remove), ("+", Operation::Install)] {
			if pkg.ends_with(modi) {
				pkg.pop();
				modifier = Some(op);
			}
		}

		let (name, version) = split_version(pkg.to_string());
		debug!("split_version: '{name}' '{version:?}'");
		let mut glob = CliPackage::new_glob(name)?.modifier(modifier);
		if let Some(ver_str) = version {
			glob.set_ver(ver_str);
		}
		globs.push(glob);
	}

	debug!("{globs:#?}");

	let arches = config.arches();
	for pkg in cache.packages(&get_sorter(config)) {
		let arch = pkg.arch();
		if !arches.iter().any(|s| s == arch) {
			continue;
		}

		let Some(cli) = globs.find_mut(pkg.name()) else {
			continue;
		};

		let pkg = pkg.filter_virtual()?;
		let version = cli.get_version(&pkg)?;
		cli.add_pkg(pkg, version, cli.modifier);
	}

	globs.check_not_found()?;
	globs.sort_by_name();

	Ok(globs)
}

pub fn regex_pkgs<'a>(config: &Config, cache: &'a Cache) -> Result<CliPackages<'a>> {
	let mut cli_pkgs = config
		.pkg_names()?
		.into_iter()
		.map(CliPackage::new_regex)
		.collect::<Result<CliPackages, _>>()?;

	let names_only = config.get_bool("names_only", false);
	let arches = config.apt.get_architectures();
	// Map packages into (Pkg, Version, DescFile)
	// Gather these so it can be sorted by the index of the DescFile
	// which makes searching 2x faster
	let mut filtered_pkgs = cache
		.packages(&get_sorter(config))
		.filter_map(|pkg| {
			if pkg.arch() != arches[0].as_str() {
				return None;
			}
			let version = pkg.versions().next()?;
			let desc = unsafe { version.translated_desc().make_safe() };
			Some((pkg, version, desc))
		})
		.collect::<Vec<_>>();

	// Some versions may not have descriptions
	filtered_pkgs.sort_by_cached_key(|p| if let Some(desc) = &p.2 { desc.index() } else { 0 });
	for (pkg, version, desc) in filtered_pkgs {
		if let Some(cli) = cli_pkgs.find_mut(pkg.name()) {
			cli.add_no_op(pkg, version);
			continue;
		}

		if names_only {
			continue;
		};

		// TODO: Fix rust-apt so that version.description uses translated desc?
		let Some(desc) = desc.and_then(|d| cache.records().desc_lookup(&d).long_desc()) else {
			continue;
		};

		if let Some(cli) = cli_pkgs.find_mut(&desc) {
			cli.add_no_op(pkg, version);
		}
	}

	cli_pkgs.check_not_found()?;

	Ok(cli_pkgs)
}
