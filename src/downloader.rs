use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Write as FmtWrite;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Error, Result};
use digest::DynDigest;
use regex::Regex;
use rust_apt::records::RecordField;
use rust_apt::{new_cache, Version};
use sha2::{Digest, Sha256, Sha512};
use tokio::fs::{self, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinSet;

use crate::colors::Theme;
use crate::config::Config;
use crate::tui;
use crate::util::NalaRegex;

/// Return the package name. Checks if epoch is needed.
fn get_pkg_name(version: &Version) -> String {
	let filename = version
		.get_record(RecordField::Filename)
		.expect("Record does not contain a filename!")
		.split_terminator('/')
		.last()
		.expect("Filename is malformed!")
		.to_string();

	if let Some(index) = version.version().find(':') {
		let epoch = format!("_{}%3a", &version.version()[..index]);
		return filename.replacen('_', &epoch, 1);
	}
	filename
}
pub struct UriFilter {
	mirrors: HashMap<String, String>,
	regex: NalaRegex,
	untrusted: HashSet<String>,
}

impl UriFilter {
	pub fn new() -> UriFilter {
		UriFilter {
			mirrors: HashMap::new(),
			regex: NalaRegex::new(),
			untrusted: HashSet::new(),
		}
	}

	/// Filter Uris from a package version.
	/// This will normalize different kinds of possible Uris
	/// Which are not http.
	fn uris<'a>(&mut self, version: &'a Version<'a>, config: &Config) -> Result<VecDeque<String>> {
		let mut filtered = VecDeque::new();

		for vf in version.version_files() {
			let pf = vf.package_file();

			if !pf.is_downloadable() {
				continue;
			}

			// Make sure the File is trusted.
			if !pf.index_file().is_trusted() {
				// Erroring is handled later if there are any untrusted URIs
				self.untrusted
					.insert(config.color(Theme::Error, version.parent().name()));
			}

			let uri = pf.index_file().archive_uri(&vf.lookup().filename());

			if uri.starts_with("file:") {
				// Sending a file path through the downloader will cause it to lock up
				// These have already been handled before the downloader runs.
				// TODO: We haven't actually handled anything yet. In python nala it happens
				// before it gets here. lol
				continue;
			}

			// We should probably consolidate this. And maybe test if mirror: works.
			if uri.starts_with("mirror+file:") || uri.starts_with("mirror:") {
				if let Some(file_match) = self.regex.mirror().captures(&uri) {
					let filename = file_match.get(1).unwrap().as_str();
					if !self.mirrors.contains_key(filename) {
						self.add_to_mirrors(&uri, filename)?;
					};

					if self
						.get_from_mirrors(version, &mut filtered, filename)
						.is_some()
					{
						continue;
					}
				}
			}
			// If none of the conditions meet then we just add it to the uris
			filtered.push_back(uri);
		}
		Ok(filtered)
	}

	/// Add the filtered Uris into the HashSet if applicable.
	fn get_from_mirrors<'a>(
		&self,
		version: &'a Version<'a>,
		uris: &mut VecDeque<String>,
		filename: &str,
	) -> Option<()> {
		// Return None if not in mirrors.
		for line in self.mirrors.get(filename)?.lines() {
			if !line.is_empty() && !line.starts_with('#') {
				uris.push_back(
					line.to_string() + "/" + &version.get_record(RecordField::Filename)?,
				);
			}
		}
		Some(())
	}

	fn add_to_mirrors(&mut self, uri: &str, filename: &str) -> Result<()> {
		self.mirrors.insert(
			filename.to_string(),
			match uri.starts_with("mirror+file:") {
				true => read_to_string(filename)?,
				false => reqwest::blocking::get("http://".to_string() + filename)?.text()?,
			},
		);
		Ok(())
	}

	/// If there are any untrusted URIs,
	/// check if we're allowed to fetch them and error otherwise.
	pub fn maybe_untrusted_error(&self, config: &Config) -> Result<()> {
		if self.untrusted.is_empty() {
			return Ok(());
		}

		config.stderr(
			Theme::Warning,
			"The Following packages cannot be authenticated!",
		);

		eprintln!(
			"  {}",
			self.untrusted
				.iter()
				.map(|s| s.to_string())
				.collect::<Vec<String>>()
				.join(", ")
		);

		if !config.apt.bool("APT::Get::AllowUnauthenticated", false) {
			bail!("Some packages were unable to be authenticated.")
		}

		config.stderr(
			Theme::Notice,
			"Configuration is set to allow installation of unauthenticated packages.",
		);
		Ok(())
	}
}

#[derive(Debug)]
pub struct Uri {
	uris: VecDeque<String>,
	size: u64,
	partial: String,
	archive: String,
	hash_type: String,
	hash_value: String,
	filename: String,
	client: reqwest::Client,
	tx: mpsc::UnboundedSender<Message>,
}

impl Uri {
	fn from_version<'a>(
		version: &'a Version<'a>,
		config: &Config,
		client: reqwest::Client,
		filter: &mut UriFilter,
		archive: &str,
		tx: mpsc::UnboundedSender<Message>,
	) -> Result<Uri> {
		let filename = get_pkg_name(version);
		let partial = format!("{archive}/partial/{filename}");
		let archive = format!("{archive}/{filename}");

		let (hash_type, hash_value) = get_hash(config, version)?;
		Ok(Uri {
			uris: filter.uris(version, config)?,
			size: version.size(),
			archive,
			partial,
			hash_type,
			hash_value,
			filename,
			client,
			tx,
		})
	}

	/// Create the File to write the download to.
	async fn open_file(&self) -> Result<File> {
		fs::File::create(&self.partial)
			.await
			.with_context(|| format!("Could not create file '{}'", self.partial))
	}

	async fn remove_file(&self) -> Result<()> {
		fs::remove_file(&self.partial)
			.await
			.with_context(|| format!("Could not remove '{}'", self.partial))
	}

	async fn move_to_archive(&self) -> Result<()> {
		fs::rename(&self.partial, &self.archive)
			.await
			.with_context(|| format!("Could not move '{}' to '{}'", self.partial, self.archive))
	}

	fn get_hasher(&self) -> Box<dyn DynDigest + Send> {
		match self.hash_type.as_str() {
			"sha256" => Box::new(Sha256::new()),
			_ => Box::new(Sha512::new()),
		}
	}

	async fn check_hash(&self, other: &str) -> Result<()> {
		if other == self.hash_value {
			return Ok(());
		}
		self.remove_file().await?;

		self.tx.send(Message::Exit)?;
		bail!("Checksum did not match for {}", &self.filename);
	}

	async fn download(
		mut self,
		mut domains: Arc<Mutex<HashMap<String, u8>>>,
		regex: Regex,
	) -> Result<Uri> {
		// This is the string URL passed to the http client
		while let Some(url) = self.uris.pop_front() {
			let Some(domain) = regex
				.captures(&url)
				.and_then(|c| c.get(1).map(|m| m.as_str()))
			else {
				continue;
			};

			self.tx.send(Message::Debug(format!(
				"Selecting {domain} for {}",
				self.filename
			)))?;

			// Lock the map so other threads can't mutate the data while this one does
			if !add_domain(domain.to_string(), &mut domains).await {
				self.tx.send(Message::Debug(format!(
					"Adding '{domain}' back to queue for {}. Connections full",
					self.filename
				)))?;

				// Too many connections to this domain.
				// Add the URL back to the queue and move to the next.
				self.uris.push_back(url);
				continue;
			}

			self.tx.send(Message::Verbose(format!("Starting: {url}")))?;
			match self.download_file(&url).await {
				Ok(hash) => {
					// Compare the hash from downloaded file against a known good hash.
					// Removes the file on disk if it doesn't match.
					self.check_hash(&hash).await?;

					// Move the good file from partial to the archive dir.
					self.move_to_archive().await?;

					self.tx.send(Message::Verbose(format!("Finished: {url}")))?;

					remove_domain(domain, &mut domains).await;
					self.tx.send(Message::Finished(self.filename.to_string()))?;
					return Ok(self);
				},
				Err(err) => {
					// Non fatal errors can continue operation.
					self.tx.send(Message::NonFatal(err))?;
					remove_domain(domain, &mut domains).await;
					continue;
				},
			}
		}
		self.tx.send(Message::Exit)?;
		bail!("No URIs could be downloaded for {}", self.filename)
	}

	/// Downloads the file and returns the hash
	pub async fn download_file(&self, url: &str) -> Result<String> {
		// Initiate http(s) connection
		let mut response = self.client.get(url).send().await?;

		// Get a mutable writer for our outfile.
		let mut writer = BufWriter::new(self.open_file().await?);
		let mut hasher = self.get_hasher();

		// Iter over the response stream and update the hasher and progress bars
		while let Some(chunk) = response.chunk().await? {
			// Send message to add to total progress bar.
			self.tx.send(Message::Update(chunk.len() as u64))?;
			hasher.update(&chunk);

			// Write the data to file
			writer.write_all(&chunk).await?;
		}

		// Build the hash string.
		let mut download_hash = String::new();
		for byte in hasher.finalize().as_ref() {
			write!(&mut download_hash, "{:02x}", byte).expect("Unable to write hash to string");
		}
		Ok(download_hash)
	}
}

// This is like to clear the terminal or something.
// There may be one other thing or something.
#[derive(Debug)]
pub enum Message {
	Exit,
	Finished(String),
	Debug(String),
	Verbose(String),
	NonFatal(Error),
	Update(u64),
}

#[derive(Debug, Eq, Hash, PartialEq)]
enum Proto {
	Http(reqwest::Url),
	Https(reqwest::Url),
	None,
}

impl Proto {
	fn new(proto: &str, domain: reqwest::Url) -> Self {
		match proto {
			"http" => Self::Http(domain),
			"https" => Self::Https(domain),
			_ => panic!("Protocol '{proto}' is not supported!"),
		}
	}

	fn maybe_proxy(&self, url: &reqwest::Url) -> Option<reqwest::Url> {
		match (self, url.scheme()) {
			// The protocol and proxy config match.
			(Proto::Http(proxy), "http") => Some(proxy.clone()),
			(Proto::Https(proxy), "https") => Some(proxy.clone()),

			// The protocol and config doesn't match.
			(Proto::Http(_), "https") => None,
			(Proto::Https(_), "http") => None,

			// For other URL schemes such as socks or ftp
			// We will just proxy them
			(Proto::Http(proxy), _) => Some(proxy.clone()),
			(Proto::Https(proxy), _) => Some(proxy.clone()),
			// This one should never actually be reached
			(Proto::None, _) => None,
		}
	}

	/// Used to get the default for all http/https if configured
	fn proxy(&self) -> Option<reqwest::Url> {
		match self {
			Proto::Http(proxy) => Some(proxy.clone()),
			Proto::Https(proxy) => Some(proxy.clone()),
			Proto::None => None,
		}
	}
}

pub fn build_proxy(config: &Config, tx: mpsc::UnboundedSender<Message>) -> Result<reqwest::Proxy> {
	let mut map: HashMap<String, Proto> = HashMap::new();

	for proto in ["http", "https"] {
		if let Some(proxy_config) = config.apt.tree(&format!("Acquire::{proto}::Proxy")) {
			// Check first for a proxy for everything
			if let Some(proxy) = proxy_config.value() {
				map.insert(
					proto.to_string(),
					Proto::new(proto, reqwest::Url::parse(&proxy)?),
				);
			}

			// Check for specific domain proxies
			if let Some(child) = proxy_config.child() {
				for node in child {
					let (Some(domain), Some(proxy)) = (node.tag(), node.value()) else {
						continue;
					};

					let lower = proxy.to_lowercase();
					if ["direct", "false"].contains(&lower.as_str()) {
						map.insert(domain, Proto::None);
						continue;
					}
					map.insert(domain, Proto::new(proto, reqwest::Url::parse(&proxy)?));
				}
			}
		}
	}

	/// Helper function to make debug messages cleaner.
	fn send_debug(
		tx: &mpsc::UnboundedSender<Message>,
		debug: bool,
		domain: &str,
		proxy: Option<&reqwest::Url>,
	) {
		if debug {
			let message = if let Some(proxy) = proxy {
				format!("Proxy for '{domain}' is '{proxy}'")
			} else {
				format!("'{domain}' Proxy is None")
			};

			tx.send(Message::Debug(message))
				.unwrap_or_else(|e| eprintln!("Error: {e}"));
		}
	}

	fn get_proxy(
		map: &HashMap<String, Proto>,
		domain: &str,
		url: &reqwest::Url,
	) -> Option<reqwest::Url> {
		// Returns None if the domain is not in the map.
		// But checking for a default is still required.
		if let Some(proto) = map.get(domain) {
			if proto == &Proto::None {
				// This domain is specifically set to not use a proxy.
				return None;
			}

			// We have to check the maybe proxy as it is based on
			// the protocol of the URL matching the config.
			// The proxy function below will not account for that.
			if let Some(proxy) = proto.maybe_proxy(url) {
				return Some(proxy);
			}
		}

		// Check for http/s default proxy.
		map.get(url.scheme())?.proxy()
	}

	let debug = config.debug();
	Ok(reqwest::Proxy::custom(move |url| {
		let domain = url.host_str()?;

		if let Some(proxy) = get_proxy(&map, domain, url) {
			send_debug(&tx, debug, domain, Some(&proxy));
			return Some(proxy);
		}
		send_debug(&tx, debug, domain, None);
		None
	}))
}

pub struct Downloader {
	client: reqwest::Client,
	uris: Vec<Uri>,
	filter: UriFilter,
	partial_dir: String,
	archive_dir: String,
	/// Used to count how many connections are open to a domain.
	/// Nala only allows 3 at a time per domain.
	domains: Arc<Mutex<HashMap<String, u8>>>,
	set: JoinSet<Result<Uri>>,
	tx: mpsc::UnboundedSender<Message>,
}

impl Downloader {
	pub fn new(proxy: reqwest::Proxy, tx: mpsc::UnboundedSender<Message>) -> Result<Downloader> {
		Ok(Downloader {
			client: reqwest::Client::builder()
				.timeout(Duration::from_secs(3))
				.proxy(proxy)
				.build()?,
			uris: vec![],
			// TODO: Make these directories configurable?
			partial_dir: "./partial".to_string(),
			archive_dir: ".".to_string(),
			filter: UriFilter::new(),
			domains: Arc::new(Mutex::new(HashMap::new())),
			set: JoinSet::new(),
			tx,
		})
	}

	fn add_version<'a>(&mut self, version: &'a Version<'a>, config: &Config) -> Result<()> {
		let uri = Uri::from_version(
			version,
			config,
			self.client.clone(),
			&mut self.filter,
			&self.archive_dir,
			self.tx.clone(),
		)?;
		self.uris.push(uri);
		Ok(())
	}

	pub async fn download(&mut self) -> Result<()> {
		// Create the partial directory
		mkdir(&self.partial_dir).await?;

		while let Some(uri) = self.uris.pop() {
			let regex = self.filter.regex.domain().clone();
			self.set.spawn(uri.download(self.domains.clone(), regex));
		}

		Ok(())
	}

	pub async fn finish(mut self) -> Result<Vec<Uri>> {
		// Finally remove the partial directory
		rmdir(&self.partial_dir).await?;

		let mut finished = vec![];
		while let Some(res) = self.set.join_next().await {
			finished.push(res??);
		}
		Ok(finished)
	}
}

#[tokio::main]
pub async fn download(config: &Config) -> Result<()> {
	let (tx, mut rx) = mpsc::unbounded_channel();

	let proxy = build_proxy(config, tx.clone())?;
	let mut downloader = Downloader::new(proxy, tx.clone())?;

	let mut not_found = vec![];
	if let Some(pkg_names) = config.pkg_names() {
		// Dedupe the pkg names. If the same pkg is given twice
		// it will be downloaded twice, and then fail when moving the file
		let mut deduped = pkg_names.clone();
		deduped.sort();
		deduped.dedup();

		let cache = new_cache!()?;
		for name in &deduped {
			if let Some(pkg) = cache.get(name) {
				let versions: Vec<Version> = pkg.versions().collect();
				for version in &versions {
					if version.is_downloadable() {
						downloader.add_version(version, config)?;
						break;
					}
					// Version wasn't downloadable
					config.stderr(
						Theme::Warning,
						&format!(
							"Can't find a source to download version '{}' of '{}'",
							version.version(),
							pkg.fullname(false)
						),
					);
				}
			} else {
				not_found.push(config.color(Theme::Notice, name));
			}
		}
	} else {
		bail!("You must specify a package")
	};

	if !not_found.is_empty() {
		for pkg in &not_found {
			config.color(Theme::Error, &format!("{pkg} not found"));
		}
		bail!("Some packages were not found.");
	}

	downloader.filter.maybe_untrusted_error(config)?;

	let mut progress = tui::NalaProgressBar::new(config)?;
	// Set the total downloads.
	let mut total = 0;
	for uri in &downloader.uris {
		total += 1;
		progress.indicatif.inc_length(uri.size)
	}

	// Start the downloads
	downloader.download().await?;

	let tick_rate = Duration::from_millis(250);
	let mut tick = Instant::now();
	let mut current = 0;
	loop {
		if current == total {
			progress.clean_up()?;
			break;
		}

		while let Ok(msg) = rx.try_recv() {
			match msg {
				Message::Update(bytes_downloaded) => progress.indicatif.inc(bytes_downloaded),
				Message::Finished(filename) => {
					current += 1;
					progress.msg = vec![
						"Total Packages:".to_string(),
						format!(" {current}/{total}, "),
						"Last Completed:".to_string(),
						format!(" {filename}"),
					];
					progress.render()?;
				},
				Message::Exit => {
					return progress.clean_up();
				},
				Message::Debug(msg) => {
					progress.print(format!("DEBUG: {msg}"))?;
				},
				Message::Verbose(msg) => {
					if config.verbose() {
						progress.print(msg)?;
					}
				},
				Message::NonFatal(err) => {
					progress.print(format!("{err}"))?;
				},
			}
		}

		if tui::poll_exit_event()? {
			progress.clean_up()?;
			downloader.set.shutdown().await;
			config.stderr(Theme::Notice, "Exiting at user request");
			return Ok(());
		}

		if tick.elapsed() >= tick_rate {
			progress.render()?;
			tick = Instant::now();
		}
	}

	let finished = downloader.finish().await?;

	println!("Downloads Complete:");
	for uri in finished {
		println!(
			"  {} was written to {}",
			config.color(Theme::Primary, &uri.filename),
			config.color(Theme::Primary, &uri.archive),
		)
	}

	Ok(())
}

/// Return the hash_type and the hash_value to be used.
fn get_hash(config: &Config, version: &Version) -> Result<(String, String)> {
	// From Debian's requirements we are not to use these for security checking.
	// https://wiki.debian.org/DebianRepository/Format#MD5Sum.2C_SHA1.2C_SHA256
	// Clients may not use the MD5Sum and SHA1 fields for security purposes,
	// and must require a SHA256 or a SHA512 field.
	// hashes = ('SHA512', 'SHA256', 'SHA1', 'MD5')

	for hash_type in ["sha512", "sha256"] {
		if let Some(hash_value) = version.hash(hash_type) {
			return Ok((hash_type.to_string(), hash_value));
		}
	}

	bail!(
		"{} {} can't be checked for integrity.\nThere are no hashes available for this package.",
		config.color(Theme::Notice, version.parent().name()),
		config.color(Theme::Notice, version.version()),
	);
}

pub async fn add_domain(domain: String, domains: &mut Arc<Mutex<HashMap<String, u8>>>) -> bool {
	let mut lock = domains.lock().await;
	let entry = lock.entry(domain).or_default();

	if *entry < 3 {
		*entry += 1;
		return true;
	}
	false
}

pub async fn remove_domain(domain: &str, domains: &mut Arc<Mutex<HashMap<String, u8>>>) {
	if let Some(entry) = domains.lock().await.get_mut(domain) {
		if *entry > 0 {
			*entry -= 1;
		}
	}
}

// Like fs::create_dir_all but it has added context for failure.
pub async fn mkdir<P: AsRef<Path> + ?Sized + std::fmt::Display>(path: &P) -> Result<()> {
	fs::create_dir_all(path)
		.await
		.with_context(|| format!("Failed to create '{path}'"))
}

pub async fn rmdir<P: AsRef<Path> + ?Sized + std::fmt::Display>(path: &P) -> Result<()> {
	fs::remove_dir(path)
		.await
		.with_context(|| format!("Failed to remove '{path}'"))
}

pub fn read_to_string<P: AsRef<Path> + ?Sized + std::fmt::Display>(path: &P) -> Result<String> {
	std::fs::read_to_string(path).with_context(|| format!("Failed to read '{path}'"))
}
