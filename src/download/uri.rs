use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use rust_apt::Version;
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

use super::Downloader;
use super::downloader::Message;
use crate::config::{Theme, color};
use crate::download::DomainMap;
use crate::fs::AsyncFs;
use crate::hashsum::{self, HashSum};
use crate::t;
use crate::util::{DOMAIN, get_pkg_name};

#[derive(Serialize)]
pub struct Uri {
	pub uris: VecDeque<String>,
	pub size: usize,
	pub archive: PathBuf,
	pub partial: PathBuf,
	pub hash: Option<HashSum>,
	pub filename: String,
	retries: usize,
	#[serde(skip)]
	bytes_downloaded: usize,
	#[serde(skip)]
	pub client: reqwest::Client,
	#[serde(skip)]
	pub tx: mpsc::UnboundedSender<Message>,
}

impl Uri {
	pub(crate) async fn required_download_size(&self) -> Result<u64> {
		if !self.archive.exists() {
			return Ok(self.size as u64);
		}

		let Some(hash) = &self.hash else {
			return Ok(self.size as u64);
		};

		let actual = HashSum::from_path(&self.archive, hash.str_type()).await?;
		Ok(if &actual == hash { 0 } else { self.size as u64 })
	}

	pub async fn from_version<'a>(
		downloader: &mut Downloader,
		version: &'a Version<'a>,
		archive: &Path,
	) -> Result<Uri> {
		let uris = downloader
			.filter
			.uris(version, archive, &downloader.client)
			.await?;
		let size = version.size() as usize;
		let filename = get_pkg_name(version);
		let hash = hashsum::get_hash(version)?;
		Ok(Self::new(downloader, uris, size, filename, Some(hash)))
	}

	pub fn new(
		downloader: &Downloader,
		uris: VecDeque<String>,
		size: usize,
		filename: String,
		hash: Option<HashSum>,
	) -> Uri {
		let archive = downloader.archive_dir.join(&filename);
		let partial = downloader.partial_dir.join(&filename);
		Self {
			uris,
			size,
			archive,
			partial,
			hash,
			filename,
			retries: 0,
			bytes_downloaded: 0,
			client: downloader.client.clone(),
			tx: downloader.tx.clone(),
		}
	}

	pub fn to_json(&self) -> Result<String> { Ok(serde_json::to_string_pretty(self)?) }

	/// Warning: If URI has None for hash_value this will not error
	/// Ensure that you make sure that it has Some(hash_string)
	async fn check_hash(&self, other: &HashSum) -> Result<()> {
		let Some(hash) = &self.hash else {
			return Ok(());
		};

		self.tx.send(Message::Debug(format!(
			"'{}':\n    Expected: {hash:?}\n    Downloaded: {other:?}",
			self.filename
		)))?;

		if other == hash {
			self.tx.send(Message::Debug("hash matched!".to_string()))?;
			return Ok(());
		}
		self.partial.remove().await?;

		self.tx.send(Message::Exit)?;
		bail!("{}", t!("download-checksum", "file" => &self.filename));
	}

	pub(crate) fn candidate_domains(&self) -> Vec<String> {
		let mut domains = Vec::new();
		for url in &self.uris {
			if let Some(domain) = DOMAIN
				.captures(url)
				.and_then(|captures| captures.get(1))
				.map(|domain| domain.as_str().to_string())
				&& !domains.contains(&domain)
			{
				domains.push(domain);
			}
		}
		domains
	}

	/// Waits until one of this package's mirrors has an available connection.
	async fn acquire_uri(&mut self, domains: &DomainMap) -> Option<(String, Arc<str>)> {
		let candidates = self.candidate_domains();
		let domain = domains.acquire(&self.filename, &candidates).await?;
		let Some(index) = self.uris.iter().position(|url| {
			DOMAIN
				.captures(url)
				.and_then(|captures| captures.get(1))
				.is_some_and(|candidate| candidate.as_str() == domain)
		}) else {
			domains.remove(&domain, &self.filename).await;
			return None;
		};
		let url = self.uris.remove(index)?;
		Some((url, domain.into()))
	}

	pub async fn download(mut self, domains: DomainMap) -> Result<Uri> {
		// First check if the file already exists on disk.
		if self.archive.exists() {
			if let Some(hash) = &self.hash {
				self.tx.send(Message::Debug(format!(
					"{:?} exists, checking hash",
					self.archive
				)))?;

				if hash == &HashSum::from_path(&self.archive, hash.str_type()).await? {
					if self.size == 0 {
						self.size = usize::try_from(std::fs::metadata(&self.archive)?.len())?;
						self.tx.send(Message::AddTotal(self.size))?;
					}
					self.tx.send(Message::Update {
						bytes: self.size,
						domain: None,
					})?;
					domains.cancel(&self.filename).await;
					self.tx.send(Message::Finished(self.filename.clone()))?;
					return Ok(self);
				}
			}
			// Async remove hangs for some reason.
			// Remove the file unconditionally since it's planned to download
			std::fs::remove_file(&self.archive)
				.with_context(|| t!("file-remove", "path" => format!("{:?}", self.archive)))?;
		}

		while let Some((url, domain)) = self.acquire_uri(&domains).await {
			self.retries = 0;

			self.tx.send(Message::Debug(t!(
				"download-select-domain",
				"domain" => &*domain,
				"file" => &self.filename
			)))?;

			while self.retries <= 3 {
				self.tx.send(Message::Verbose(t!(
					"download-start",
					"uri" => &url,
					"retries" => self.retries
				)))?;
				self.bytes_downloaded = 0;
				match self.download_file(&url).await {
					Ok(hash) => {
						domains.remove(&domain, &self.filename).await;

						// Compare the hash from downloaded file against a known
						// good hash. Removes the file on disk if it
						// doesn't match.
						self.check_hash(&hash).await?;

						// Move the good file from partial to the archive dir.
						self.partial.rename(&self.archive).await?;
						self.tx.send(Message::Verbose(t!(
							"download-finished",
							"uri" => &url
						)))?;

						self.tx.send(Message::Finished(self.filename.clone()))?;
						return Ok(self);
					},
					Err(err) => {
						// Non fatal errors can continue operation.
						self.retries += 1;
						self.tx.send(Message::NonFatal {
							error: err,
							bytes: self.bytes_downloaded,
							domain: domain.clone(),
						})?;
						continue;
					},
				}
			}
			domains.remove(&domain, &self.filename).await;
		}
		self.tx.send(Message::Exit)?;
		bail!("{}", t!("download-no-uris", "file" => &self.filename))
	}

	/// Downloads the file and returns the hash
	pub async fn download_file(&mut self, url: &str) -> Result<HashSum> {
		let domain = DOMAIN
			.captures(url)
			.and_then(|captures| captures.get(1))
			.map(|domain| Arc::<str>::from(domain.as_str()));

		// Initiate http(s) connection
		let mut response = self
			.client
			.get(url)
			.send()
			.await
			.context(t!("download-get"))?
			.error_for_status()
			.with_context(|| t!("download-request-failed", "uri" => url))?;

		if self.size == 0
			&& let Some(size) = response
				.content_length()
				.and_then(|size| usize::try_from(size).ok())
		{
			self.size = size;
			self.tx.send(Message::AddTotal(size))?;
		}

		// Get a mutable writer for our outfile.
		let mut writer = self.partial.open_writer().await?;

		let default_hash = HashSum::Sha512(String::new());
		let hash_type = self.hash.as_ref().unwrap_or(&default_hash).str_type();
		let mut hasher = hashsum::get_hasher(hash_type)?;

		// Iter over the response stream and update the hasher and progress bars
		while let Some(chunk) = response
			.chunk()
			.await
			.with_context(|| t!("download-stream-failed", "uri" => url))?
		{
			// Send message to add to total progress bar.
			self.tx.send(Message::Update {
				bytes: chunk.len(),
				domain: domain.clone(),
			})?;
			self.bytes_downloaded += chunk.len();
			hasher.update(&chunk);

			// Write the data to file
			writer.write_all(&chunk).await?;
		}
		writer.flush().await?;

		if self.size == 0 {
			self.size = self.bytes_downloaded;
			self.tx.send(Message::AddTotal(self.size))?;
		}

		Ok(hasher.finalize_hashsum())
	}
}

pub struct UriFilter {
	mirrors: HashMap<String, String>,
	pub untrusted: HashSet<String>,
}

impl UriFilter {
	pub fn new() -> UriFilter {
		UriFilter {
			mirrors: HashMap::new(),
			untrusted: HashSet::new(),
		}
	}

	pub fn add_untrusted(&mut self, item: &str) {
		self.untrusted
			.insert(color::color!(Theme::Error, item).to_string());
	}

	/// Filter Uris from a package version.
	/// This will normalize different kinds of possible Uris
	/// Which are not http.
	async fn uris<'a>(
		&mut self,
		version: &'a Version<'a>,
		archive: &Path,
		client: &reqwest::Client,
	) -> Result<VecDeque<String>> {
		let mut filtered = VecDeque::new();

		for vf in version.version_files() {
			let pf = vf.package_file();

			if !pf.is_downloadable() {
				continue;
			}

			// Make sure the File is trusted.
			if !pf.index_file().is_trusted() {
				// Erroring is handled later if there are any untrusted URIs
				self.add_untrusted(version.parent().name());
			}

			let package_filename = vf.lookup().filename();
			let archive_base = pf.index_file().archive_uri("");
			let uri = pf.index_file().archive_uri(&package_filename);

			// Any real files should be copied into the Archive directory for
			// use
			if let Some(path) = uri.strip_prefix("file:").map(Path::new) {
				let Some(filename) = path.file_name() else {
					bail!("{}", t!("download-filename", "path" => format!("{path:?}")))
				};
				path.cp(archive.join(filename)).await?;
			}

			if let Some(location) = mirror_location(&archive_base)
				&& let Some(package_path) = uri.strip_prefix(&archive_base)
			{
				if !self.mirrors.contains_key(&location) {
					self.add_to_mirrors(client, &location).await?;
				};

				if let Some(mirrors) = self.mirrors.get(&location) {
					add_mirror_uris(mirrors, package_path.trim_start_matches('/'), &mut filtered);
					continue;
				}
			}
			// If none of the conditions meet then we just add it to the uris
			filtered.push_back(uri);
		}
		Ok(filtered)
	}

	async fn add_to_mirrors(&mut self, client: &reqwest::Client, location: &str) -> Result<()> {
		self.mirrors.insert(
			location.to_string(),
			match location.strip_prefix("file:") {
				Some(path) => Path::new(path).read_string().await?,
				None => {
					client
						.get(location)
						.send()
						.await?
						.error_for_status()?
						.text()
						.await?
				},
			},
		);
		Ok(())
	}
}

fn add_mirror_uris(mirrors: &str, package_filename: &str, uris: &mut VecDeque<String>) {
	for line in mirrors.lines() {
		if let Some(mirror) = line.split_ascii_whitespace().next()
			&& !mirror.starts_with('#')
		{
			uris.push_back(format!(
				"{}/{package_filename}",
				mirror.trim_end_matches('/')
			));
		}
	}
}

fn mirror_location(uri: &str) -> Option<String> {
	let location = uri.trim_end_matches('/');

	if let Some(location) = location.strip_prefix("mirror://") {
		return Some(format!("http://{location}"));
	}

	location
		.strip_prefix("mirror+")
		.filter(|location| {
			location.starts_with("http://")
				|| location.starts_with("https://")
				|| location.starts_with("file:")
		})
		.map(str::to_string)
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::*;
	use crate::config::Config;

	#[test]
	fn mirror_sources_are_normalized_and_expanded() {
		for (source, expected) in [
			("mirror://host/list", "http://host/list"),
			("mirror+http://host/list", "http://host/list"),
			("mirror+https://host/list", "https://host/list"),
			("mirror+file:/tmp/list", "file:/tmp/list"),
		] {
			assert_eq!(
				mirror_location(&format!("{source}/")).as_deref(),
				Some(expected)
			);
		}

		let filename = "pool/main/z/zstd/zstd_1.5.7%2bdfsg-1_arm64.deb";
		let mut uris = VecDeque::new();
		add_mirror_uris(
			"# comment\n\n https://one/\tpriority:1\nhttp://two",
			filename,
			&mut uris,
		);

		assert_eq!(
			uris,
			VecDeque::from([
				"https://one/pool/main/z/zstd/zstd_1.5.7%2bdfsg-1_arm64.deb".to_string(),
				"http://two/pool/main/z/zstd/zstd_1.5.7%2bdfsg-1_arm64.deb".to_string(),
			])
		);
	}

	#[tokio::test]
	async fn scheduler_rotates_first_choice_across_mirrors() {
		let downloader = Downloader::new(&Config::default()).unwrap();
		let domains = DomainMap::default();
		let mirrors = ["one", "two", "three", "four"];
		let urls = mirrors.map(|mirror| format!("https://{mirror}.example/pkg.deb"));
		let mut selected = HashMap::<String, usize>::new();
		for package in 0..mirrors.len() * super::super::DOMAIN_CONNECTION_LIMIT {
			let filename = format!("pkg-{package}.deb");
			let mut uri = Uri::new(
				&downloader,
				urls.iter().cloned().collect(),
				0,
				filename,
				None,
			);
			domains
				.register(&uri.filename, &uri.candidate_domains())
				.await;
			let (_, domain) =
				tokio::time::timeout(Duration::from_millis(50), uri.acquire_uri(&domains))
					.await
					.unwrap()
					.unwrap();
			*selected.entry(domain.to_string()).or_default() += 1;
			domains.remove(&domain, &uri.filename).await;
		}

		for mirror in mirrors {
			assert_eq!(
				selected.get(&format!("{mirror}.example")),
				Some(&super::super::DOMAIN_CONNECTION_LIMIT)
			);
		}
	}
}
