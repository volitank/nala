use anyhow::{bail, Result};
use rust_apt::{new_cache, Package};

use crate::config::Config;
use crate::debug;
use crate::t;

fn get_origin_codename(pkg: Option<Package>) -> Option<(String, String)> {
	let pkg_file = pkg?.candidate()?.package_files().next()?;

	Some((
		pkg_file.origin()?.to_string(),
		pkg_file.codename()?.to_string(),
	))
}

pub(super) fn detect_release(config: &Config) -> Result<(String, String, String)> {
	for distro in ["debian", "ubuntu", "devuan"] {
		if let Some(value) = config.get_str(distro) {
			debug!("Distro '{distro} {value}' passed on CLI");
			let distro = distro.to_string();
			let keyring = format!("/usr/share/keyrings/{distro}-archive-keyring.gpg");
			return Ok((distro, value.to_lowercase(), keyring));
		}
	}

	let cache = new_cache!()?;

	for keyring in [
		"devuan-keyring",
		"debian-archive-keyring",
		"ubuntu-keyring",
		"apt",
	] {
		if let Some((origin, codename)) = get_origin_codename(cache.get(keyring)) {
			debug!("Distro/Release Found on '{keyring}'");
			// devuan-archive-keyring.gpg
			// ubuntu-archive-keyring.gpg
			// debian-archive-keyring.gpg
			let distro = origin.to_lowercase();
			let keyring = format!("/usr/share/keyrings/{distro}-archive-keyring.gpg");
			return Ok((distro, codename.to_lowercase(), keyring));
		}
	}
	bail!("{}", t!("fetch-release-detect"));
}

pub(super) fn get_component(config: &Config, distro: &str) -> Result<String> {
	let mut component = "main".to_string();
	if distro == "devuan" || distro == "debian" {
		if config.get_bool("non_free", false) {
			component += " contrib non-free"
		}
		return Ok(component);
	}

	if distro == "ubuntu" {
		// It's Ubuntu, you probably don't care about foss
		return Ok(component + " restricted universe multiverse");
	}
	bail!("{}", t!("fetch-unsupported", "distro" => distro))
}
