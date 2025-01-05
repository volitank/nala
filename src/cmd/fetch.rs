use std::collections::HashSet;
use std::fs;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use reqwest::Client;
use rust_apt::tagfile::{parse_tagfile, TagSection};
use rust_apt::{new_cache, Package};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::Duration;

use crate::config::{Config, Paths};
use crate::util::{sudo_check, DOMAIN, UBUNTU_COUNTRY, UBUNTU_URL};
use crate::{debug, tui};

fn get_origin_codename(pkg: Option<Package>) -> Option<(String, String)> {
	let pkg_file = pkg?.candidate()?.package_files().next()?;

	Some((
		pkg_file.origin()?.to_string(),
		pkg_file.codename()?.to_string(),
	))
}

fn detect_release(config: &Config) -> Result<(String, String, String)> {
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
	bail!("There was an issue detecting release.");
}

fn get_component(config: &Config, distro: &str) -> Result<String> {
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
	bail!("{distro} is unsupported.")
}

fn domain_from_list(line: &str) -> Option<String> {
	if line.starts_with('#') || line.is_empty() {
		return None;
	}
	regex_string(line)
}

fn regex_string(line: &str) -> Option<String> {
	Some(DOMAIN.captures(line)?.get(1)?.as_str().to_string())
}

fn parse_sources(config: &Config) -> Result<HashSet<String>> {
	let mut sources = HashSet::new();

	// Read and extract domains from the main sources.list file
	let main = config.get_file(&Paths::SourceList);
	for line in fs::read_to_string(&main)
		.with_context(|| format!("Failed to read {main}"))?
		.lines()
	{
		if let Some(domain) = domain_from_list(line) {
			sources.insert(domain);
		}
	}

	// Parts could be either .list or .sources
	let parts = config.get_path(&Paths::SourceParts);
	for file in
		fs::read_dir(&parts).with_context(|| format!("Failed to read '{}'", parts.display()))?
	{
		let path = file?.path();
		if path.is_dir() {
			continue;
		}

		let filename = path.to_string_lossy();

		// Don't consider nala sources as it'll be overwritten
		if filename.ends_with("nala.sources") {
			continue;
		}

		// Continue if the file isn't .sources or .list
		if !filename.ends_with(".sources") && !filename.ends_with(".list") {
			continue;
		}

		let data = fs::read_to_string(&path)
			.with_context(|| format!("Failed to read '{}'", path.display()))?;

		if filename.ends_with(".sources") {
			for section in parse_tagfile(&data)? {
				let enabled = section.get_default("Enabled", "yes").to_lowercase();

				// These sources are disabled. So we can ignore them
				if ["no", "false", "0"].contains(&enabled.as_str()) {
					continue;
				}

				let Some(uris) = section.get("URIs") else {
					continue;
				};

				for uri in uris.split_whitespace() {
					if uri.is_empty() {
						continue;
					}

					if let Some(domain) = regex_string(uri) {
						sources.insert(domain);
					}
				}
			}
			continue;
		}

		if filename.ends_with(".list") {
			for line in data.as_str().lines() {
				if let Some(domain) = domain_from_list(line) {
					sources.insert(domain);
				}
			}
		}
	}
	Ok(sources)
}

fn fetch_mirrors(
	config: &Config,
	countries: &Option<HashSet<String>>,
	distro: &str,
) -> Result<HashSet<String>> {
	let mut net_select = HashSet::new();
	if distro == "debian" {
		let response =
			reqwest::blocking::get("https://mirror-master.debian.org/status/Mirrors.masterlist")?
				.text()?;

		let tagfile = rust_apt::tagfile::parse_tagfile(&response).unwrap();
		let arches = config.apt.get_architectures();

		for section in tagfile {
			if let Some(url) = debian_url(countries, &section, &arches) {
				net_select.insert(url);
			}
		}
	} else if distro == "ubuntu" {
		let response =
			reqwest::blocking::get("https://launchpad.net/ubuntu/+archivemirrors-rss")?.text()?;

		let mirrors = response.split("<item>");
		for mirror in mirrors {
			if let Some(url) = ubuntu_url(config, countries, mirror) {
				net_select.insert(url);
			}
		}
	} else if distro == "devuan" {
		let response =
			reqwest::blocking::get("https://pkgmaster.devuan.org/mirror_list.txt")?.text()?;

		let tagfile = rust_apt::tagfile::parse_tagfile(&response).unwrap();
		for section in tagfile {
			if let Some(url) = devuan_url(countries, &section) {
				net_select.insert(url);
			}
		}
	}
	Ok(net_select)
}

#[tokio::main]
async fn check_non_free(
	config: &Config,
	chosen: &[String],
	mut component: String,
	release: &str,
) -> Result<String> {
	let mut set = JoinSet::new();

	if !config.get_bool("non_free", false) {
		return Ok(component);
	}

	let client = Client::builder().timeout(Duration::from_secs(5)).build()?;

	for url in chosen.iter() {
		set.spawn(
			client
				.get(format!("{url}/dists/{release}/non-free-firmware/"))
				.send(),
		);
	}

	let mut values = Vec::with_capacity(set.len());
	// Run all of the futures.
	while let Some(res) = set.join_next().await {
		values.push(res.is_ok_and(|r| r.is_ok_and(|r| r.error_for_status().is_ok())));
	}

	// Debatable that we should add separate entries if it exists or not
	if values.iter().all(|b| *b) {
		component += " non-free-firmware";
		return Ok(component);
	}
	Ok(component)
}

#[tokio::main]
/// Score the mirrors and provide a progress bar.
async fn score_handler(
	config: &Config,
	mirror_strings: HashSet<String>,
	release: &str,
) -> Result<Vec<(String, u128)>> {
	// Setup Progress Bar
	let mut pb = tui::NalaProgressBar::new(config, false)?;
	pb.indicatif.set_length(mirror_strings.len() as u64);

	let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
	let semp = Arc::new(Semaphore::new(30));
	// If we decide we want more information during this portion
	// for something like verbose/debug we probably need an mpsc
	// let (tx, mut rx) = mpsc::unbounded_channel();
	let mut set = JoinSet::new();
	for url in &mirror_strings {
		set.spawn(net_select_score(
			client.clone(),
			semp.clone(),
			config.get_bool("https_only", false),
			url.strip_suffix('/').unwrap_or(url).to_string(),
			release.to_string(),
		));
	}

	// Get the results from scoring.
	let mut score = vec![];
	while let Some(res) = set.join_next().await {
		if let Ok(Ok(response)) = res {
			pb.dg.push_str("Finished: ", response.0.to_string());
			score.push(response)
		}
		pb.indicatif.inc(1);
		pb.render()?;
		if tui::poll_exit_event()? {
			pb.clean_up()?;
			std::process::exit(1);
		}
	}
	pb.clean_up()?;

	// Move FetchScore out of its Arc and then return the final vec.
	score.sort_by_key(|k| k.1);
	Ok(score)
}

/// Score the url with https and http depending on config.
async fn net_select_score(
	client: Client,
	semp: Arc<Semaphore>,
	https_only: bool,
	url: String,
	release: String,
) -> Result<(String, u128)> {
	let sem = semp.acquire_owned().await?;
	let https = url.replace("http://", "https://");

	if let Ok(response) = fetch_release(&client, &https, &release).await {
		return Ok(response);
	}

	// We don't need to check http if it's https-only or if its only https
	if !https_only && !url.contains("https://") {
		if let Ok(response) = fetch_release(&client, &url, &release).await {
			return Ok(response);
		}
	}

	drop(sem);
	bail!("Could not get to '{url}'")
}

/// Fetch the release file and handle errors
///
/// This will return Some(String) if its NOT successful
/// None is successful
async fn fetch_release(client: &Client, base_url: &str, release: &str) -> Result<(String, u128)> {
	// TODO: Should we verify the release file is proper?
	let before = std::time::Instant::now();
	client
		.get(format!("{base_url}/dists/{release}/Release"))
		.send()
		.await?
		.error_for_status()?;
	let after = before.elapsed().as_millis();
	Ok((base_url.to_string(), after))
}

fn debian_url(
	countries: &Option<HashSet<String>>,
	section: &TagSection,
	arches: &[String],
) -> Option<String> {
	// If there are countries provided
	if let Some(hash_set) = countries {
		let country = section.get("Country")?.split_whitespace().next()?;

		// If it doesn't match any provided return None
		if !hash_set.contains(country) {
			return None;
		}
	}

	// There were either no countries provided or there was a match
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

fn ubuntu_url(
	config: &Config,
	countries: &Option<HashSet<String>>,
	mirror: &str,
) -> Option<String> {
	if mirror.contains("<title>Ubuntu Archive Mirrors Status</title>") {
		return None;
	}

	let only_ports = config
		.apt
		.get_architectures()
		.iter()
		.any(|arch| arch != "amd64" && arch != "i386");

	if let Some(hash_set) = countries {
		if !hash_set.contains(UBUNTU_COUNTRY.captures(mirror)?.get(1)?.as_str()) {
			return None;
		}
	}

	let url = UBUNTU_URL.captures(mirror)?.get(1)?.as_str();
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

fn devuan_url(countries: &Option<HashSet<String>>, section: &TagSection) -> Option<String> {
	if !section.get("Protocols")?.contains("HTTP") {
		return None;
	}

	if let Some(hash_set) = countries {
		for country in hash_set {
			if !section.get("CountryCode")?.contains(country) {
				return None;
			}
		}
	}

	Some(format!("http://{}/devuan", section.get("BaseURL")?.trim()))
}

/// The entry point for the `fetch` command.
pub fn fetch(config: &Config) -> Result<()> {
	sudo_check(config)?;

	let (distro, release, keyring) = detect_release(config)?;
	debug!("Detected '{distro}:{release}'");

	let component = get_component(config, &distro)?;
	debug!("Initial component '{component}'");

	let countries: Option<HashSet<String>> = match config.countries() {
		Some(values) => {
			let mut hash_set = HashSet::new();
			for value in values {
				hash_set.insert(value.to_uppercase());
			}
			Some(hash_set)
		},
		None => None,
	};

	// Get the current sources on disk to not create duplicates
	let sources = parse_sources(config)?;
	debug!("Sources on disk {sources:#?}");

	// Get the mirrors
	let mut net_select = fetch_mirrors(config, &countries, &distro)?;
	debug!("NetSelect size '{}'", net_select.len());

	// Remove domains that are already defined on disk
	let mut remove = HashSet::new();
	for mirror in &net_select {
		for source in &sources {
			if mirror.contains(source) {
				remove.insert(mirror.to_string());
			}
		}
	}
	net_select.retain(|n| !remove.contains(n));
	debug!("NetSelect Dedupe Size '{}'", net_select.len());

	// Score the mirrors
	let scored = score_handler(config, net_select, &release)?;
	debug!("Scored Mirrors '{}'", scored.len());

	if scored.is_empty() {
		bail!("Nala was unable to find any mirrors.")
	}

	// Only run the TUI if --auto is not on
	let chosen = if config.auto().is_some() {
		debug!("Auto mode, not starting TUI");
		scored.into_iter().map(|(s, _)| s).collect()
	} else {
		debug!("Interactive mode, starting TUI");
		let terminal = tui::init_terminal()?;
		let chosen = tui::fetch::App::new(config, scored).run(terminal)?;
		tui::restore_terminal()?;
		chosen
	};

	if chosen.is_empty() {
		bail!("No mirrors were selected.")
	}

	debug!("Building Nala sources file");
	let mut nala_sources = "# Sources file built for nala\n\n".to_string();
	// Types: deb deb-src
	// URIs: https://deb.volian.org/volian/
	// Suites: scar
	// Components: main
	// Signed-By: /usr/share/keyrings/volian-archive-scar-unstable.gpg
	nala_sources += if config.get_bool("sources", false) {
		"Types: deb\n"
	} else {
		"Types: deb deb-src\n"
	};

	nala_sources += "URIs: ";
	for (i, mirror) in chosen.iter().enumerate() {
		if config.auto().is_some_and(|auto| i + 1 > auto as usize) {
			break;
		}
		if i > 0 {
			nala_sources += "      ";
		}
		nala_sources += &format!("{mirror}\n");
	}
	nala_sources += &format!("Suites: {release}\n");
	nala_sources += &format!(
		"Components: {}\n",
		check_non_free(config, &chosen, component, &release)?
	);
	nala_sources += &format!("Signed-By: {keyring}\n");

	debug!("Writing the following to file:\n\n{nala_sources}");

	let file = config.get_file(&Paths::NalaSources);
	fs::write(&file, nala_sources)?;
	println!("Sources have been written to {file}");
	Ok(())
}
