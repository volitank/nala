use std::collections::BTreeMap;

use anyhow::Result;
use rust_apt::{new_cache, Cache, Package, PackageFile, PinnedPackage, Version};
use serde::Serialize;

use crate::config::{color, keys, Config};
use crate::glob;

const DPKG_STATUS_FILE: &str = "Debian dpkg status file";

#[derive(Serialize)]
struct PolicySource {
	priority: i32,
	source: String,
}

#[derive(Serialize)]
struct PackagePolicyVersion {
	version: String,
	priority: i32,
	installed: bool,
	candidate: bool,
	sources: Vec<PolicySource>,
}

#[derive(Serialize)]
struct PackagePolicy {
	package: String,
	installed: Option<String>,
	candidate: Option<String>,
	versions: Vec<PackagePolicyVersion>,
}

#[derive(Serialize)]
struct GlobalPolicyFile {
	priority: i32,
	source: String,
	release: Option<String>,
	site: Option<String>,
	downloadable: bool,
}

#[derive(Serialize)]
struct PolicyPin {
	name: String,
	version: String,
	priority: i32,
}

#[derive(Serialize)]
struct GlobalPolicy {
	package_files: Vec<GlobalPolicyFile>,
	pinned_packages: Vec<PolicyPin>,
}

fn ver_string(ver: Option<&str>) -> String {
	ver.map(|value| color::secondary!(value))
		.unwrap_or_else(|| "none".to_string())
}

fn status_file(file: &PackageFile<'_>) -> bool {
	file.index_type()
		.is_some_and(|kind| kind == DPKG_STATUS_FILE)
}

fn include_global_file(file: &PackageFile<'_>) -> bool {
	file.is_downloadable() || status_file(file)
}

fn release_line(file: &PackageFile<'_>) -> Option<String> {
	if status_file(file) {
		return Some("release a=now".to_string());
	}

	let parts = [
		("o", file.origin()),
		("a", file.archive()),
		("n", file.codename()),
		("l", file.label()),
		("c", file.component()),
		("b", file.arch()),
	]
	.into_iter()
	.filter_map(|(key, value)| {
		value
			.filter(|value| !value.is_empty())
			.map(|value| format!("{key}={value}"))
	})
	.collect::<Vec<_>>();

	if parts.is_empty() {
		return None;
	}

	Some(format!("release {}", parts.join(",")))
}

fn collect_version_sources(ver: &Version<'_>) -> Vec<PolicySource> {
	let mut by_index: BTreeMap<u64, PolicySource> = BTreeMap::new();

	for file in ver.package_files() {
		by_index
			.entry(file.index())
			.or_insert_with(|| PolicySource {
				priority: file.priority(),
				source: file.index_file().describe(true).trim().to_string(),
			});
	}

	let mut sources = by_index.into_values().collect::<Vec<_>>();
	sources.sort_by(|a, b| {
		b.priority
			.cmp(&a.priority)
			.then_with(|| a.source.cmp(&b.source))
	});
	sources
}

fn collect_package_policy(pkg: &Package<'_>) -> PackagePolicy {
	let installed = pkg.installed();
	let candidate = pkg.candidate();
	let installed_version = installed.as_ref().map(|ver| ver.version().to_string());
	let candidate_version = candidate.as_ref().map(|ver| ver.version().to_string());

	let mut versions = pkg.versions().collect::<Vec<_>>();
	versions.sort_by(|a, b| {
		b.priority_with_files(true)
			.cmp(&a.priority_with_files(true))
			.then_with(|| b.cmp(a))
	});

	let versions = versions
		.into_iter()
		.map(|ver| PackagePolicyVersion {
			version: ver.version().to_string(),
			priority: ver.priority_with_files(true),
			installed: installed.as_ref().is_some_and(|value| value == &ver),
			candidate: candidate.as_ref().is_some_and(|value| value == &ver),
			sources: collect_version_sources(&ver),
		})
		.collect();

	PackagePolicy {
		package: pkg.fullname(true).to_string(),
		installed: installed_version,
		candidate: candidate_version,
		versions,
	}
}

fn collect_global_file(file: &PackageFile<'_>) -> Option<GlobalPolicyFile> {
	if !include_global_file(file) {
		return None;
	}

	Some(GlobalPolicyFile {
		priority: file.priority(),
		source: file.index_file().describe(true).trim().to_string(),
		release: release_line(file),
		site: file
			.site()
			.filter(|site| !site.is_empty())
			.map(ToString::to_string),
		downloadable: file.is_downloadable(),
	})
}

fn collect_pinned_package(pin: &PinnedPackage) -> PolicyPin {
	PolicyPin {
		name: pin.name.to_string(),
		version: pin.version.to_string(),
		priority: pin.priority,
	}
}

fn collect_global_policy(cache: &Cache) -> GlobalPolicy {
	let package_files = cache
		.package_files()
		.filter_map(|file| collect_global_file(&file))
		.collect::<Vec<_>>();

	let mut pinned_packages = cache
		.pinned_packages()
		.map(|pin| collect_pinned_package(&pin))
		.collect::<Vec<_>>();
	pinned_packages.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.version.cmp(&b.version)));

	GlobalPolicy {
		package_files,
		pinned_packages,
	}
}

fn print_package_policy(policy: &PackagePolicy) {
	println!("{}:", color::primary!(&policy.package));
	println!("  Installed: {}", ver_string(policy.installed.as_deref()));
	println!("  Candidate: {}", ver_string(policy.candidate.as_deref()));
	println!("  Version table:");

	if policy.versions.is_empty() {
		println!("    {}", color::secondary!("No versions."));
		return;
	}

	for ver in &policy.versions {
		let marker = if ver.installed { "***" } else { "   " };

		println!(
			"  {marker} {} {}",
			color::secondary!(&ver.version),
			ver.priority,
		);

		for source in &ver.sources {
			println!("      {:>3} {}", source.priority, source.source);
		}
	}
}

fn print_global_file(file: &GlobalPolicyFile) {
	println!(" {:>3} {}", file.priority, file.source);

	if let Some(release) = &file.release {
		println!("     {release}");
	}

	if let Some(site) = &file.site {
		println!("     origin {site}");
	}
}

fn print_pinned_package(pin: &PolicyPin) {
	println!("     {} -> {} ({})", pin.name, pin.version, pin.priority);
}

fn policy_global_machine(cache: &Cache) -> Result<()> {
	let policy = collect_global_policy(cache);
	println!("{}", serde_json::to_string_pretty(&policy)?);
	Ok(())
}

fn policy_global(cache: &Cache) {
	let policy = collect_global_policy(cache);

	println!("Package files:");
	for file in &policy.package_files {
		print_global_file(file);
	}

	println!("Pinned packages:");
	for pin in &policy.pinned_packages {
		print_pinned_package(pin);
	}
}

fn policy_machine(packages: Vec<Package<'_>>) -> Result<()> {
	let packages = packages
		.into_iter()
		.map(|pkg| collect_package_policy(&pkg))
		.collect::<Vec<_>>();

	println!("{}", serde_json::to_string_pretty(&packages)?);
	Ok(())
}

pub fn policy(config: &Config) -> Result<()> {
	let cache = new_cache!()?;
	let pkg_names = config.get_vec(keys::PKG_NAMES);
	if pkg_names.is_none_or(|names| names.is_empty()) {
		if config.get_bool(keys::MACHINE, false) {
			return policy_global_machine(&cache);
		}

		policy_global(&cache);
		return Ok(());
	}

	let selection = glob::pkgs_with_modifiers(config.pkg_names()?, config, &cache)?;
	let (packages, missing) = selection.into_packages_and_missing();
	if config.get_bool(keys::MACHINE, false) {
		glob::log_missing_notices(&missing);
		return policy_machine(packages);
	}

	for (idx, pkg) in packages.into_iter().enumerate() {
		if idx > 0 {
			println!();
		}

		let policy = collect_package_policy(&pkg);
		print_package_policy(&policy);
	}

	glob::log_missing_notices(&missing);

	Ok(())
}
