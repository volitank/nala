use std::collections::HashSet;

use anyhow::Result;
use reqwest::Client;
use rust_apt::tagfile::{parse_tagfile, TagSection};
use tokio::time::Duration;

use crate::config::Config;

const DEBIAN_MIRRORS_URL: &str = "https://mirror-master.debian.org/status/Mirrors.masterlist";
const UBUNTU_MIRRORS_URL: &str =
	"https://api.launchpad.net/devel/ubuntu/archive_mirrors?ws.size=300";
const DEVUAN_MIRRORS_URL: &str = "https://pkgmaster.devuan.org/mirror_list.txt";

pub(super) async fn fetch_mirrors(
	config: &Config,
	countries: &Option<HashSet<String>>,
	distro: &str,
) -> Result<HashSet<String>> {
	let client = Client::builder().timeout(Duration::from_secs(15)).build()?;

	match distro {
		"debian" => {
			let arches = config.apt.get_architectures();
			fetch_tagfile_mirrors(
				&client,
				DEBIAN_MIRRORS_URL,
				countries,
				|section| debian_url(section, &arches),
			)
			.await
		},
		"ubuntu" => fetch_ubuntu_mirrors(config, countries, &client).await,
		"devuan" => {
			fetch_tagfile_mirrors(&client, DEVUAN_MIRRORS_URL, countries, devuan_url).await
		},
		_ => Ok(HashSet::new()),
	}
}

async fn fetch_ubuntu_mirrors(
	config: &Config,
	countries: &Option<HashSet<String>>,
	client: &Client,
) -> Result<HashSet<String>> {
	let mut net_select = HashSet::new();
	let only_ports = config
		.apt
		.get_architectures()
		.iter()
		.any(|arch| arch != "amd64" && arch != "i386");

	let mut next = Some(UBUNTU_MIRRORS_URL.to_string());
	while let Some(url) = next {
		let response = client
			.get(url)
			.send()
			.await?
			.error_for_status()?
			.json::<serde_json::Value>()
			.await?;

		for mirror in response["entries"].as_array().into_iter().flatten() {
			if let Some(url) = ubuntu_url(countries, only_ports, mirror) {
				net_select.insert(url);
			}
		}

		next = response["next_collection_link"].as_str().map(str::to_string);
	}

	Ok(net_select)
}

async fn fetch_tagfile_mirrors(
	client: &Client,
	url: &str,
	countries: &Option<HashSet<String>>,
	mirror_url: impl Fn(&TagSection) -> Option<String>,
) -> Result<HashSet<String>> {
	let response = client
		.get(url)
		.send()
		.await?
		.error_for_status()?
		.text()
		.await?;
	let mut mirrors = HashSet::new();

	for section in parse_tagfile(&response)? {
		if !country_allowed(countries, &section) {
			continue;
		}

		if let Some(url) = mirror_url(&section) {
			mirrors.insert(url);
		}
	}

	Ok(mirrors)
}

fn debian_url(section: &TagSection, arches: &[String]) -> Option<String> {
	let mirror_arches = section.get("Archive-architecture")?;
	if arches.iter().all(|arch| mirror_arches.contains(arch)) {
		return Some(format!(
			"http://{}{}",
			section.get("Site")?,
			section.get("Archive-http")?
		));
	}
	None
}

fn country_allowed(countries: &Option<HashSet<String>>, section: &TagSection) -> bool {
	let Some(countries) = countries else {
		return true;
	};

	if let Some(country_codes) = section.get("CountryCode") {
		return country_codes
			.split('|')
			.map(str::trim)
			.any(|country| countries.contains(country));
	}

	section
		.get("Country")
		.and_then(|country| country.split_whitespace().next())
		.is_some_and(|country| countries.contains(country))
}

fn ubuntu_url(
	countries: &Option<HashSet<String>>,
	only_ports: bool,
	mirror: &serde_json::Value,
) -> Option<String> {
	if mirror["enabled"].as_bool() != Some(true) {
		return None;
	}

	if mirror["status"].as_str() != Some("Official") {
		return None;
	}

	if let Some(hash_set) = countries {
		let country = mirror["country_link"].as_str()?.rsplit('/').next()?;
		if !hash_set.contains(country) {
			return None;
		}
	}

	let url = mirror["http_base_url"]
		.as_str()
		.or_else(|| mirror["https_base_url"].as_str())
		.or_else(|| mirror["base_url"].as_str())?;
	let is_ports = url.contains("ubuntu-ports");

	// Don't return non ports if we only want ports
	if only_ports && !is_ports {
		return None;
	}

	// Don't return ports if we don't want only_ports
	if !only_ports && is_ports {
		return None;
	}

	Some(url.to_string())
}

fn devuan_url(section: &TagSection) -> Option<String> {
	Some(format!("http://{}/devuan", section.get("BaseURL")?.trim()))
}
