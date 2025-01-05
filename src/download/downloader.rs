use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Error, Result};
use rust_apt::{new_cache, Version};
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinSet;

use super::{proxy, Uri, UriFilter};
use crate::config::{color, Config, Paths, Theme};
use crate::fs::AsyncFs;
use crate::hashsum::HashSum;
use crate::{debug, dprog, info, tui, warn};

pub async fn download(config: &Config) -> Result<()> {
	// Set download directory to the cwd.
	config.apt.set(Paths::Archive.path(), "./");

	let mut downloader = Downloader::new(config)?;
	let mut not_found = vec![];

	let cache = new_cache!()?;
	let pkg_names = config.pkg_names()?;
	let archive = config.get_path(&Paths::Archive);
	for name in &pkg_names {
		if let Some(pkg) = cache.get(name) {
			let versions: Vec<Version> = pkg.versions().collect();
			for version in &versions {
				if version.is_downloadable() {
					downloader.add_version(version, &archive).await?;
					break;
				}
				warn!(
					"Can't find a source to download version '{}' of '{}'",
					version.version(),
					pkg.fullname(false)
				);
			}
		} else {
			not_found.push(color::color!(Theme::Notice, name).to_string());
		}
	}

	if !not_found.is_empty() {
		for pkg in &not_found {
			color::color!(Theme::Error, &format!("{pkg} not found"));
		}
		bail!("Some packages were not found.");
	}

	let finished = downloader.run(config, true).await?;

	println!("Downloads Complete:");
	for uri in finished {
		println!(
			"  {} was written to {}",
			color::primary!(&uri.filename),
			color::primary!(&uri.archive.to_string_lossy()),
		)
	}

	Ok(())
}

/// If there are any untrusted URIs,
/// check if we're allowed to fetch them and error otherwise.
///
/// Each String in Vec<String> is a pkg_name or url
/// ["apt", "nala", "fastfetch"]
pub fn untrusted_error(config: &Config, untrusted: Vec<String>) -> Result<()> {
	if untrusted.is_empty() {
		return Ok(());
	}
	warn!("The Following packages cannot be authenticated!");
	eprintln!("  {}", untrusted.join(", "));

	if !config.allow_unauthenticated() {
		bail!(format!(
			"Some packages were unable to be authenticated.\n  If you're sure use {}",
			color::color!(Theme::Notice, "--allow-unauthenticated")
		));
	}

	info!("Configuration is set to allow installation of unauthenticated packages.");
	Ok(())
}

// This is like to clear the terminal or something.
// There may be one other thing or something.
#[derive(Debug)]
pub enum Message {
	Exit,
	Finished,
	Debug(String),
	Verbose(String),
	NonFatal((Error, usize)),
	Update(usize),
}

pub struct Downloader {
	pub(crate) client: reqwest::Client,
	uris: Vec<Uri>,
	pub(crate) filter: UriFilter,
	pub(crate) archive_dir: PathBuf,
	pub(crate) partial_dir: PathBuf,
	/// Used to count how many connections are open to a domain.
	/// Nala only allows 3 at a time per domain.
	domains: Arc<Mutex<HashMap<String, u8>>>,
	set: JoinSet<Result<Uri>>,
	pub(crate) tx: mpsc::UnboundedSender<Message>,
	rx: mpsc::UnboundedReceiver<Message>,
}

impl Downloader {
	pub fn new(config: &Config) -> Result<Downloader> {
		let archive_dir = config.get_path(&Paths::Archive);
		let partial_dir = archive_dir.join("partial");

		let (tx, rx) = mpsc::unbounded_channel();
		let proxy = proxy::build_proxy(config, tx.clone())?;

		Ok(Downloader {
			client: reqwest::Client::builder()
				.timeout(Duration::from_secs(30))
				.proxy(proxy)
				.build()?,
			uris: vec![],
			// TODO: Make these directories configurable?
			archive_dir,
			partial_dir,
			filter: UriFilter::new(),
			domains: Arc::new(Mutex::new(HashMap::new())),
			set: JoinSet::new(),
			tx,
			rx,
		})
	}

	pub async fn add_version<'a>(
		&mut self,
		version: &'a Version<'a>,
		archive: &Path,
	) -> Result<()> {
		let uri = Uri::from_version(self, version, archive).await?;
		self.uris.push(uri);
		Ok(())
	}

	/// This method ingests URLs from the command line to download
	pub async fn add_from_cmdline(&mut self, cli_uri: &str) -> Result<()> {
		let mut parser = cli_uri.split_terminator(":");

		let Some(protocol) = parser.next() else {
			bail!("No protocol was defined")
		};

		// Rebuild the string to maintain order
		let Some(uri) = parser.next().map(|u| format!("{protocol}:{u}")) else {
			bail!("No uri was defined")
		};

		// sha512 d500faf8b2b9ee3a8fbc6a18f966076ed432894cd4d17b42514ffffac9ee81ce
		// 945610554a11df24ded152569b77693c57c7967dd71f644af3066bf79a923bfe
		//
		// sha256 a694f44fa05fff6d00365bf23217d978841b9e7c8d7f48e80864df08cebef1a8
		// md5 b9ef863f210d170d282991ad1e0676eb
		// sha1 d1f34ed00dea59f886b9b99919dfcbbf90d69e15
		let hash = if let Some(hashsum) = parser.next() {
			Some(HashSum::from_str_len(hashsum.len(), hashsum.to_string())?)
		} else {
			warn!("No Hash Found for '{uri}'");
			None
		};

		let response = self.client.head(&uri).send().await?.error_for_status()?;

		// Check headers for the size of the download
		let headers = response.headers();

		debug!("URL Headers for {uri} {headers:#?}");
		let Some(content_len) = response.headers().get("content-length") else {
			bail!("content-length does not exist in {headers:#?}");
		};

		let size = content_len
			.to_str()
			.with_context(|| format!("Converting content-len to &str {headers:#?}"))?
			.parse::<usize>()
			.with_context(|| format!("Parsing content-len to usize {headers:#?}"))?;

		let Some(filename) = uri.split_terminator("/").last().map(|s| s.to_string()) else {
			bail!("'{uri}' is malformed!");
		};

		self.uris
			.push(Uri::new(self, VecDeque::from([uri]), size, filename, hash));

		Ok(())
	}

	pub fn uris(&self) -> &Vec<Uri> { &self.uris }

	pub async fn download(&mut self) -> Result<()> {
		// Create the partial directory
		self.partial_dir.mkdir().await?;

		while let Some(uri) = self.uris.pop() {
			self.set.spawn(uri.download(self.domains.clone()));
		}

		Ok(())
	}

	async fn finish(mut self, rm_partial: bool) -> Result<Vec<Uri>> {
		// Finally remove the partial directory
		if rm_partial {
			self.partial_dir.remove_recurse().await?;
		}

		let mut finished = vec![];
		while let Some(res) = self.set.join_next().await {
			finished.push(res??);
		}
		Ok(finished)
	}

	pub async fn run(mut self, config: &Config, rm_partial: bool) -> Result<Vec<Uri>> {
		if config.debug() {
			for uri in self.uris() {
				debug!("{}", uri.to_json()?);
			}
		}
		// TODO: This is correct, but it is also likely very inefficient.
		// Decide if it's worth refactoring.
		// I don't believe we'll have many perf issues here
		self.uris()
			.iter()
			// Iterate uris and get the filenames of all the ones who do not have hashes
			.filter(|&uri| uri.hash.is_none())
			.map(|uri| uri.filename.to_string())
			// Collect so filter_map runs before for_each due to mut and immutable borrows
			.collect::<Vec<_>>()
			.into_iter()
			// Add all the filenames without hashes into the filter
			.for_each(|filename| self.filter.add_untrusted(&filename));

		if !self.filter.untrusted.is_empty() {
			untrusted_error(config, self.filter.untrusted.iter().cloned().collect())?;
		}

		let mut progress = tui::NalaProgressBar::new(config, false)?;
		// Set the total downloads.
		let mut total = 0;
		for uri in &self.uris {
			total += 1;
			progress.indicatif.inc_length(uri.size as u64)
		}

		// Start the downloads
		self.download().await?;

		let tick_rate = Duration::from_millis(150);
		let mut tick = Instant::now();
		let mut current = 0;
		'outer: loop {
			if current == total {
				progress.clean_up()?;
				break;
			}

			while let Ok(msg) = self.rx.try_recv() {
				match msg {
					Message::Update(bytes_downloaded) => {
						progress.indicatif.inc(bytes_downloaded as u64)
					},
					Message::Finished => {
						current += 1;
					},
					Message::Exit => {
						progress.clean_up()?;
						break 'outer;
					},
					Message::Debug(msg) => {
						dprog!(config, progress, "downloader", "{msg}");
					},
					Message::Verbose(msg) => {
						if config.verbose() {
							progress.print(&msg)?;
						}
					},
					Message::NonFatal((err, size)) => {
						progress.print(&format!("Error: {err:?}"))?;
						progress
							.indicatif
							.set_position(progress.length() - size as u64)
					},
				}
			}

			if tui::poll_exit_event()? {
				progress.clean_up()?;
				self.set.shutdown().await;
				info!("Exiting at user request");
				return Ok(vec![]);
			}

			if tick.elapsed() >= tick_rate {
				progress
					.dg
					.clear()
					.push_str("Packages:", format!(" {current}/{total}"))
					.push_str("Connections:", format!(" {:?}", self.domains.lock().await));

				progress.render()?;
				tick = Instant::now();
			}
		}

		let finished = self.finish(rm_partial).await?;
		if finished.is_empty() {
			bail!("Downloads Failed")
		}
		Ok(finished)
	}
}
