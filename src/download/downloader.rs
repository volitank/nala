use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Error, Result, bail};
use rust_apt::{Version, new_cache};
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use super::{DomainMap, Uri, UriFilter, proxy};
use crate::config::{Config, Paths, Theme, color};
use crate::fs::AsyncFs;
use crate::hashsum::HashSum;
use crate::progress::Progress;
use crate::terminal::poll_exit_event;
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

// This is like to clear the terminal or something.
// There may be one other thing or something.
#[derive(Debug)]
pub enum Message {
	Exit,
	Finished,
	Debug(String),
	Verbose(String),
	NonFatal((Error, usize)),
	AddTotal(usize),
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
			domains: DomainMap::new(),
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

		let mut progress = Progress::with_tui_lines(config, false, 16)?;
		// Set the total downloads.
		let mut total = 0usize;
		for uri in &self.uris {
			total += 1;
			progress.inc_length(uri.size as u64)
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
					Message::AddTotal(size) => progress.inc_length(size as u64),
					Message::Update(bytes_downloaded) => progress.inc(bytes_downloaded as u64),
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
					Message::NonFatal((err, bytes_downloaded)) => {
						progress.print(&t!("download-error", "error" => format!("{err:?}")))?;
						progress.dec(bytes_downloaded as u64)
					},
				}
			}

			if poll_exit_event()? {
				progress.clean_up()?;
				self.set.shutdown().await;
				info!("{}", t!("download-exit"));
				return Ok(vec![]);
			}

			if tick.elapsed() >= tick_rate {
				progress.set_info(vec![(t!("download-items"), format!("{current}/{total}"))]);
				progress.set_panels(self.domains.panels().await);

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
