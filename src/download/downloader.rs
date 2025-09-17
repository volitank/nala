use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Error, Result};
use ratatui::layout::{Constraint, Layout};
use rust_apt::Version;
use tokio::sync::{mpsc, Mutex};
use tokio::task::JoinSet;

use super::{proxy, Uri, UriFilter};
use crate::config::{color, Config, Paths, Theme};
use crate::fs::AsyncFs;
use crate::hashsum::HashSum;
use crate::{dprog, tui};

/// If there are any untrusted URIs,
/// check if we're allowed to fetch them and error otherwise.
///
/// Each String in Vec<String> is a pkg_name or url
/// ["apt", "nala", "fastfetch"]
pub fn untrusted_error(config: &Config, untrusted: Vec<String>) -> Result<()> {
	if untrusted.is_empty() {
		return Ok(());
	}
	crate::warning!("The Following packages cannot be authenticated!");
	eprintln!("  {}", untrusted.join(", "));

	if !config.allow_unauthenticated() {
		bail!(format!(
			"Some packages were unable to be authenticated.\n  If you're sure use {}",
			color::color!(Theme::Notice, "--allow-unauthenticated")
		));
	}

	crate::notice!("Configuration is set to allow installation of unauthenticated packages.");
	Ok(())
}

// This is like to clear the terminal or something.
// There may be one other thing or something.
#[derive(Debug)]
pub enum Message {
	Start((String, usize)),
	Exit,
	Finished(String),
	Debug(String),
	Verbose(String),
	NonFatal((Error, usize)),
	Update((String, usize)),
}

pub struct Downloader {
	pub(crate) client: reqwest::Client,
	uris: Vec<Uri>,
	pub(crate) filter: UriFilter,
	pub(crate) archive_dir: PathBuf,
	pub(crate) partial_dir: PathBuf,
	/// Used to count how many connections are open to a domain.
	/// Nala only allows 3 at a time per domain.
	pub(crate) domains: Arc<Mutex<HashMap<String, u8>>>,
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
			crate::warning!("No Hash Found for '{uri}'");
			None
		};

		let response = self.client.head(&uri).send().await?.error_for_status()?;

		// Check headers for the size of the download
		let headers = response.headers();

		crate::debug!("URL Headers for {uri} {headers:#?}");
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
				crate::debug!("{}", uri.to_json()?);
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

		let mut term = tui::Term::init_viewport(16)?;
		let mut progress = tui::NalaProgressBar::new(config)?;
		let mut dg = tui::progress::DisplayGroup::new();

		// Set the total bytes to download.
		for uri in &self.uris {
			progress.inc_length(uri.size as u64)
		}

		let total: u16 = self.uris().len().try_into().unwrap();

		// Start the downloads
		self.download().await?;

		let tick_rate = Duration::from_millis(250);
		let mut tick = Instant::now();

		let mut current = 0;

		'outer: loop {
			if current == total {
				progress.clean_up(&mut term)?;
				break;
			}

			while let Ok(msg) = self.rx.try_recv() {
				match msg {
					Message::Start((name, total)) => {
						// progress.dg.push(PkgProgress::new(name, total as
						// u64));
					},
					Message::Update((name, inc)) => {
						// progress.dg.update(&name, inc as u64);
						progress.pb.inc(inc as u64)
					},
					Message::Finished(filename) => {
						// progress.dg.remove(&filename);
						current += 1;
					},
					Message::Exit => {
						progress.clean_up(&mut term)?;
						break 'outer;
					},
					Message::Debug(msg) => {
						dprog!(config, &mut term, progress, "downloader", "{msg}");
					},
					Message::Verbose(msg) => {
						if config.verbose() {
							progress.print(&mut term, &msg)?;
						}
					},
					Message::NonFatal((err, size)) => {
						progress.print(&mut term, &format!("Error: {err:?}"))?;
						progress.pb.set_position(progress.length() - size as u64)
					},
				}
			}

			if tui::poll_exit_event()? {
				progress.clean_up(&mut term)?;
				self.set.shutdown().await;
				crate::notice!("Exiting at user request");
				return Ok(vec![]);
			}

			if tick.elapsed() >= tick_rate {
				if progress.disabled {
					continue;
				}

				for (k, v) in [
					("Packages:", format!(" {current}/{total}")),
					("Connections:", format!(" {:?}", self.domains.lock().await)),
					("Total:", progress.current_total()),
					(
						"PerSec:",
						format!(" {}/s", progress.unit.str(progress.per_sec() as u64)),
					),
				] {
					dg.push_str(k.to_string(), v);
				}

				let _ = term.term.draw(|f| {
					let block = crate::tui::vblock(&config.color);
					let [_, info, bar] = Layout::vertical([
						Constraint::Fill(100),
						Constraint::Min(0),
						Constraint::Length(1),
					])
					.areas(block.inner(f.area()));

					f.render_widget(block, f.area());
					f.render_widget(&mut dg, info);
					f.render_widget(&progress, bar);
				})?;

				// self.draw(&mut term, &config.color, &mut progress, &mut dg)?;
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
