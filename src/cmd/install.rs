use anyhow::{bail, Result};
use rust_apt::new_cache;
use rust_apt::util::show_broken_pkg;

use crate::cmd::Operation;
use crate::config::Config;
use crate::deb::DebFile;
use crate::download::Downloader;
use crate::glob::CliPackage;
use crate::util::sudo_check;
use crate::{debug, glob, info};

/// Sort command line pkgs and download http pkgs
///
/// Return the first Vec is packages to lookup in the cache, the 2nd is DebFiles
pub async fn split_local(config: &Config) -> Result<(Vec<String>, Vec<DebFile>)> {
	let mut http_pkgs = vec![];
	let mut deb_files = vec![];
	let mut cache_pkgs = vec![];

	for pkg in config.pkg_names()? {
		if pkg.starts_with("http") {
			http_pkgs.push(pkg);
			continue;
		}

		// Treat it as pkg name if it isn't .deb
		if !pkg.ends_with(".deb") {
			cache_pkgs.push(pkg);
			continue;
		}

		// All else are local
		deb_files.push(DebFile::new(pkg).await?);
	}

	if !http_pkgs.is_empty() {
		let mut downloader = Downloader::new(config)?;
		for pkg in http_pkgs {
			downloader.add_from_cmdline(&pkg).await?;
		}

		for uri in downloader.run(config, true).await? {
			if config.verbose() {
				println!("Downloaded: {:?}", uri.archive)
			}
			deb_files.push(DebFile::new(uri.archive.to_string_lossy().into()).await?);
		}
	}
	Ok((cache_pkgs, deb_files))
}

pub async fn mark_cli_pkgs(config: &mut Config, operation: Operation) -> Result<()> {
	sudo_check(config)?;

	let (cache_pkgs, deb_files) = split_local(config).await?;
	let deb_paths: Vec<&str> = deb_files.iter().map(|deb| deb.path.as_str()).collect();
	let cache = new_cache!(&deb_paths)?;

	let mut packages = glob::pkgs_with_modifiers(cache_pkgs, config, &cache)?;

	// Fetch the correct local .deb and version from the cache
	for deb in deb_files {
		let Some(pkg) = cache.get(deb.name()) else {
			continue;
		};

		info!(
			"Selecting Package '{}' instead of '{}'",
			pkg.name(),
			deb.path
		);

		let Some(ver) = pkg.get_version(deb.version()) else {
			bail!(
				"Could not find Version '{}' for Package '{}'",
				deb.version(),
				pkg.name()
			)
		};
		ver.set_candidate();
		packages.push(CliPackage::new_glob(pkg.name().to_string())?.with_pkg(pkg, ver))
	}

	packages.mark(&cache, operation, config.get_bool("purge", false))?;

	if let Err(err) = cache.resolve(false) {
		debug!("Broken Count: {}", cache.depcache().broken_count());
		for pkg in cache.iter() {
			if let Some(broken) = show_broken_pkg(&cache, &pkg, false) {
				eprintln!("{broken}");
			};
		}
		bail!(err);
	}

	crate::summary::commit(cache, config).await
}
