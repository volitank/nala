use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use rust_apt::records::RecordField;
use rust_apt::Version;
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc, Mutex};

use super::downloader::Message;
use super::Downloader;
use crate::config::{color, Theme};
use crate::fs::AsyncFs;
use crate::hashsum::{self, HashSum};
use crate::util::{get_pkg_name, DOMAIN, MIRROR};

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
	pub client: reqwest::Client,
	#[serde(skip)]
	pub tx: mpsc::UnboundedSender<Message>,
}

impl Uri {
	pub async fn from_version<'a>(
		downloader: &mut Downloader,
		version: &'a Version<'a>,
		archive: &Path,
	) -> Result<Uri> {
		let uris = downloader.filter.uris(version, archive).await?;
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
		bail!("Checksum did not match for {}", &self.filename);
	}

	pub async fn download(mut self, mut domains: Arc<Mutex<HashMap<String, u8>>>) -> Result<Uri> {
		// First check if the file already exists on disk.
		if self.archive.exists() {
			if let Some(hash) = &self.hash {
				self.tx.send(Message::Debug(format!(
					"{:?} exists, checking hash",
					self.archive
				)))?;

				if hash == &HashSum::from_path(&self.archive, hash.str_type()).await? {
					self.tx.send(Message::Update(self.size))?;
					self.tx.send(Message::Finished)?;
					return Ok(self);
				}
			}
			// Async remove hangs for some reason.
			// Remove the file unconditionally since it's planned to download
			std::fs::remove_file(&self.archive)
				.with_context(|| format!("Unable to remove {:?}", self.archive))?;
		}

		// This is the string URL passed to the http client
		while let Some(url) = self.uris.pop_front() {
			self.retries = 0;
			let Some(domain) = DOMAIN
				.captures(&url)
				.and_then(|c| c.get(1).map(|m| m.as_str()))
			else {
				continue;
			};

			// Lock the map so other threads can't mutate the data while this one does
			if !add_domain(domain.to_string(), &mut domains).await {
				// Too many connections to this domain.
				// Add the URL back to the queue and move to the next.
				self.uris.push_back(url);
				continue;
			}

			self.tx.send(Message::Debug(format!(
				"Selecting {domain} for {}",
				self.filename
			)))?;

			while self.retries <= 3 {
				self.tx.send(Message::Verbose(format!(
					"Starting: {url}, Retries: {}",
					self.retries
				)))?;
				match self.download_file(&url).await {
					Ok(hash) => {
						// Compare the hash from downloaded file against a known good hash.
						// Removes the file on disk if it doesn't match.
						self.check_hash(&hash).await?;

						// Move the good file from partial to the archive dir.
						self.partial.rename(&self.archive).await?;
						self.tx.send(Message::Verbose(format!("Finished: {url}")))?;

						remove_domain(domain, &mut domains).await;
						self.tx.send(Message::Finished)?;
						return Ok(self);
					},
					Err(err) => {
						// Non fatal errors can continue operation.
						self.retries += 1;
						self.tx.send(Message::NonFatal((err, self.size)))?;
						remove_domain(domain, &mut domains).await;
						continue;
					},
				}
			}
		}
		self.tx.send(Message::Exit)?;
		bail!("No URIs could be downloaded for {}", self.filename)
	}

	/// Downloads the file and returns the hash
	pub async fn download_file(&self, url: &str) -> Result<HashSum> {
		// Initiate http(s) connection
		let mut response = self.client.get(url).send().await.context("Get")?;

		// Get a mutable writer for our outfile.
		let mut writer = self.partial.open_writer().await?;

		let default_hash = HashSum::Sha512(String::new());
		let hash_type = self.hash.as_ref().unwrap_or(&default_hash).str_type();
		let mut hasher = hashsum::get_hasher(hash_type)?;

		// Iter over the response stream and update the hasher and progress bars
		while let Some(chunk) = response
			.chunk()
			.await
			.with_context(|| format!("Unable to stream data from '{url}'"))?
		{
			// Send message to add to total progress bar.
			self.tx.send(Message::Update(chunk.len()))?;
			hasher.update(&chunk);

			// Write the data to file
			writer.write_all(&chunk).await?;
		}
		writer.flush().await?;

		HashSum::from_str(hash_type, hashsum::bytes_to_hex_string(&hasher.finalize()))
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

			let uri = pf.index_file().archive_uri(&vf.lookup().filename());

			// Any real files should be copied into the Archive directory for use
			if let Some(path) = uri.strip_prefix("file:").map(Path::new) {
				let Some(filename) = path.file_name() else {
					bail!("{path:?} Does not have a valid filename!")
				};
				path.cp(archive.join(filename)).await?;
			}

			// We should probably consolidate this. And maybe test if mirror: works.
			if uri.starts_with("mirror+file:") || uri.starts_with("mirror:") {
				if let Some(file_match) = MIRROR.captures(&uri) {
					let filename = file_match.get(1).unwrap().as_str();
					if !self.mirrors.contains_key(filename) {
						self.add_to_mirrors(&uri, filename).await?;
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

	async fn add_to_mirrors(&mut self, uri: &str, filename: &str) -> Result<()> {
		self.mirrors.insert(
			filename.to_string(),
			match uri.starts_with("mirror+file:") {
				true => Path::new(filename).read_string().await?,
				false => reqwest::blocking::get("http://".to_string() + filename)?.text()?,
			},
		);
		Ok(())
	}
}
