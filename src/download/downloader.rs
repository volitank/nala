use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Error, Result, bail};
use indexmap::{IndexMap, IndexSet};
use nix::sys::statvfs::statvfs;
use rust_apt::{Version, new_cache};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::{DOMAIN_CONNECTION_LIMIT, DomainMap, Uri, UriFilter, proxy};
use crate::config::{Config, Paths, Theme, color};
use crate::fs::AsyncFs;
use crate::hashsum::HashSum;
use crate::progress::{MirrorProgress, Progress, ProgressMessage};
use crate::terminal::poll_exit_event;
use crate::util::DOMAIN;
use crate::{debug, dprog, info, t, warn};

pub async fn download(config: &Config) -> Result<()> {
	// Set download directory to the cwd.
	config.apt.set(Paths::Archive.path(), "./");

	let mut downloader = Downloader::new(config)?;
	let mut not_found = false;

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
					"{}",
					t!(
						"download-source-missing",
						"version" => version.version(),
						"package" => pkg.fullname(false)
					)
				);
			}
		} else {
			not_found = true;
		}
	}

	if not_found {
		bail!("{}", t!("download-some-missing"));
	}

	let finished = downloader.run(config, true).await?;

	println!("{}", t!("download-complete"));
	for uri in finished {
		println!(
			"  {}",
			t!(
				"download-written",
				"package" => color::primary!(&uri.filename),
				"path" => color::primary!(&uri.archive.to_string_lossy())
			)
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
	warn!("{}", t!("download-auth-warning"));
	eprintln!("  {}", untrusted.join(", "));

	if !config.allow_unauthenticated() {
		bail!(
			"{}",
			t!(
				"download-auth-required",
				"switch" => color::color!(Theme::Notice, "--allow-unauthenticated")
			)
		);
	}

	info!("{}", t!("download-auth-allowed"));
	Ok(())
}

#[derive(Debug)]
pub enum Message {
	Exit,
	Finished(String),
	Debug(String),
	Verbose(String),
	NonFatal {
		error: Error,
		bytes: usize,
		domain: Arc<str>,
	},
	AddTotal(usize),
	Update {
		bytes: usize,
		domain: Option<Arc<str>>,
	},
}

#[derive(Default)]
struct MirrorTransfer {
	bytes: u64,
	elapsed: Duration,
	active_since: Option<Instant>,
	seen: bool,
}

impl MirrorTransfer {
	fn add(&mut self, bytes: usize) {
		self.active_since.get_or_insert_with(Instant::now);
		self.bytes = self.bytes.saturating_add(bytes as u64);
		self.seen = true;
	}

	fn remove(&mut self, bytes: usize) { self.bytes = self.bytes.saturating_sub(bytes as u64) }

	fn rate(&mut self, active: bool) -> Option<u64> {
		if !active && let Some(started) = self.active_since.take() {
			self.elapsed += started.elapsed();
		}

		if !self.seen {
			return None;
		}
		let elapsed = (self.elapsed
			+ self
				.active_since
				.map_or(Duration::ZERO, |started| started.elapsed()))
		.as_secs_f64();
		Some(if elapsed <= 0.0 {
			self.bytes
		} else {
			(self.bytes as f64 / elapsed).ceil() as u64
		})
	}
}

struct MirrorTransfers(IndexMap<String, MirrorTransfer>);

impl MirrorTransfers {
	fn new(domains: &[String]) -> Self {
		Self(
			domains
				.iter()
				.cloned()
				.map(|domain| (domain, MirrorTransfer::default()))
				.collect(),
		)
	}

	fn add(&mut self, domain: &str, bytes: usize) {
		self.0.entry(domain.to_string()).or_default().add(bytes)
	}

	fn remove(&mut self, domain: &str, bytes: usize) {
		if let Some(transfer) = self.0.get_mut(domain) {
			transfer.remove(bytes);
		}
	}

	fn rows(&mut self, active: &[(String, usize)]) -> Vec<MirrorProgress> {
		active
			.iter()
			.map(|(domain, active)| {
				let (rate, seen) = self.0.get_mut(domain).map_or((None, false), |transfer| {
					(transfer.rate(*active > 0), transfer.seen)
				});
				MirrorProgress::new(domain.clone(), *active, DOMAIN_CONNECTION_LIMIT, rate, seen)
			})
			.collect()
	}
}

pub struct Downloader {
	pub(crate) client: reqwest::Client,
	uris: Vec<Uri>,
	pub(crate) filter: UriFilter,
	pub(crate) archive_dir: PathBuf,
	pub(crate) partial_dir: PathBuf,
	/// Used to count how many connections are open to a domain.
	/// Nala limits concurrent connections to each domain.
	domains: DomainMap,
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
				.connect_timeout(Duration::from_secs(30))
				.read_timeout(Duration::from_secs(120))
				.proxy(proxy)
				.build()?,
			uris: vec![],
			// TODO: Make these directories configurable?
			archive_dir,
			partial_dir,
			filter: UriFilter::new(),
			domains: DomainMap::default(),
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
	pub fn add_from_cmdline(&mut self, cli_uri: &str) -> Result<()> {
		let (uri, filename, hash) = parse_cli_uri(cli_uri)?;

		if hash.is_none() {
			warn!("{}", t!("download-hash-missing", "uri" => &uri));
		}

		self.uris
			.push(Uri::new(self, VecDeque::from([uri]), 0, filename, hash));

		Ok(())
	}

	pub fn uris(&self) -> &Vec<Uri> { &self.uris }

	fn configured_domains(&self) -> Vec<String> {
		let mut domains = IndexSet::new();
		for url in self.uris.iter().flat_map(|uri| &uri.uris) {
			if let Some(domain) = DOMAIN.captures(url).and_then(|captures| captures.get(1)) {
				domains.insert(domain.as_str().to_string());
			}
		}
		domains.into_iter().collect()
	}

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

		self.archive_dir.mkdir().await?;
		let mut required = 0_u64;
		for uri in &self.uris {
			required = required.saturating_add(uri.required_download_size().await?);
		}
		let fs = statvfs(&self.archive_dir)?;
		let available = (fs.blocks_available() as u64).saturating_mul(fs.fragment_size() as u64);
		if required > available {
			bail!(
				"{}",
				t!(
					"download-no-space",
					"path" => self.archive_dir.display().to_string(),
					"required" => config.unit_str(required),
					"available" => config.unit_str(available)
				)
			);
		}

		let configured_domains = self.configured_domains();
		for uri in &self.uris {
			self.domains
				.register(&uri.filename, &uri.candidate_domains())
				.await;
		}
		let mut mirror_transfers = MirrorTransfers::new(&configured_domains);
		let mut progress = Progress::download(config, configured_domains.len())?;
		// Set the total downloads.
		let mut total = 0usize;
		for uri in &self.uris {
			total += 1;
			progress.inc_length(uri.size as u64)
		}
		progress.set_items(0, total);
		progress.set_mirrors(mirror_transfers.rows(&self.domains.active().await));

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
					Message::AddTotal(size) => progress.inc_length(size as u64),
					Message::Update { bytes, domain } => {
						if let Some(domain) = domain {
							progress.inc(bytes as u64);
							mirror_transfers.add(&domain, bytes);
						} else {
							progress.inc_cached(bytes as u64);
						}
					},
					Message::Finished(filename) => {
						current += 1;
						progress.set_message(ProgressMessage::new(
							format!("{}: ", t!("progress-last-completed")),
							vec![filename],
						));
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
					Message::NonFatal {
						error,
						bytes,
						domain,
					} => {
						progress.print(&t!("download-error", "error" => format!("{error:?}")))?;
						progress.dec(bytes as u64);
						mirror_transfers.remove(&domain, bytes);
					},
				}
			}

			if poll_exit_event()? {
				progress.clean_up()?;
				self.set.shutdown().await;
				bail!("{}", t!("download-exit"));
			}

			if tick.elapsed() >= tick_rate {
				progress.set_items(current, total);
				progress.set_mirrors(mirror_transfers.rows(&self.domains.active().await));

				progress.render()?;
				tick = Instant::now();
			}
		}

		let finished = self.finish(rm_partial).await?;
		if finished.is_empty() {
			bail!("{}", t!("download-failed"))
		}
		Ok(finished)
	}
}

fn parse_cli_uri(cli_uri: &str) -> Result<(String, String, Option<HashSum>)> {
	let (uri, hash) = match cli_uri.rsplit_once(':') {
		Some((uri, digest))
			if matches!(digest.len(), 64 | 128)
				&& digest.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
		{
			(
				uri,
				Some(HashSum::from_str_len(digest.len(), digest.to_string())?),
			)
		},
		_ => (cli_uri, None),
	};

	let Ok(parsed) = reqwest::Url::parse(uri) else {
		bail!("{}", t!("download-malformed", "uri" => cli_uri));
	};
	if !matches!(parsed.scheme(), "http" | "https") {
		bail!("{}", t!("download-malformed", "uri" => cli_uri));
	}

	let Some(filename) = parsed
		.path_segments()
		.and_then(|mut segments| segments.next_back())
		.filter(|filename| !filename.is_empty())
		.map(str::to_string)
	else {
		bail!("{}", t!("download-malformed", "uri" => cli_uri));
	};

	Ok((parsed.to_string(), filename, hash))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn mirror_average_stops_while_idle() {
		let mut transfer = MirrorTransfer::default();
		transfer.add(1024);
		transfer.active_since = Some(Instant::now() - Duration::from_secs(1));

		let rate = transfer.rate(false);
		let elapsed = transfer.elapsed;
		assert_eq!(transfer.rate(false), rate);
		assert_eq!(transfer.elapsed, elapsed);

		transfer.add(1024);
		assert!(transfer.active_since.is_some());
	}

	#[test]
	fn command_line_uri_preserves_ports_and_extracts_the_filename() {
		let (uri, filename, hash) =
			parse_cli_uri("http://[::1]:8080/packages/demo.deb?source=test").unwrap();

		assert_eq!(uri, "http://[::1]:8080/packages/demo.deb?source=test");
		assert_eq!(filename, "demo.deb");
		assert_eq!(hash, None);
	}

	#[test]
	fn command_line_uri_accepts_a_trailing_sha256() {
		let digest = "a".repeat(64);
		let input = format!("https://example.test/demo.deb:{digest}");
		let (uri, filename, hash) = parse_cli_uri(&input).unwrap();

		assert_eq!(uri, "https://example.test/demo.deb");
		assert_eq!(filename, "demo.deb");
		assert_eq!(hash, Some(HashSum::Sha256(digest)));
	}
}
