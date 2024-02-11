use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use bytes::Bytes;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
	disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use digest::DynDigest;
use indicatif::{FormattedDuration, ProgressBar};
use once_cell::sync::OnceCell;
use ratatui::layout::Flex;
use ratatui::prelude::*;
use ratatui::style::Stylize;
use ratatui::widgets::block::Title;
use ratatui::widgets::*;
use regex::{Regex, RegexBuilder};
use reqwest::Response;
use rust_apt::cache::Cache;
use rust_apt::new_cache;
use rust_apt::package::Version;
use rust_apt::records::RecordField;
use rust_apt::util::{terminal_width, unit_str, NumSys};
use sha2::{Digest, Sha256, Sha512};
use tokio::fs;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{Mutex, MutexGuard};
use tokio::task::JoinSet;
use tokio::time::Duration;

use crate::config::Config;

pub struct MirrorRegex {
	mirror: OnceCell<Regex>,
	mirror_file: OnceCell<Regex>,
}

impl MirrorRegex {
	fn new() -> Self {
		MirrorRegex {
			mirror: OnceCell::new(),
			mirror_file: OnceCell::new(),
		}
	}

	fn mirror(&self) -> Result<&Regex> {
		self.mirror.get_or_try_init(|| {
			Ok(RegexBuilder::new(r"mirror://(.*?/.*?)/")
				.case_insensitive(true)
				.build()?)
		})
	}

	fn mirror_file(&self) -> Result<&Regex> {
		self.mirror_file.get_or_try_init(|| {
			Ok(RegexBuilder::new(r"mirror\+file:(/.*?)/pool")
				.case_insensitive(true)
				.build()?)
		})
	}
}

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

// #[derive(Clone, Debug)]
pub struct Uri {
	uris: Vec<String>,
	archive: String,
	destination: String,
	hash_type: String,
	hash_value: String,
	filename: String,
	progress: Progress,
	is_finished: bool,
	crash: bool,
	errors: Vec<String>,
}

impl Uri {
	async fn from_version<'a>(
		version: &'a Version<'a>,
		cache: &Cache,
		config: &Config,
		downloader: &mut Downloader,
		archive: String,
	) -> Result<Arc<Mutex<Uri>>> {
		let progress = Progress::new(version.size());

		let (hash_type, hash_value) = get_hash(config, version)?;

		Ok(Arc::new(Mutex::new(Uri {
			uris: downloader.filter_uris(version, cache, config).await?,
			archive: archive.clone() + &get_pkg_name(version),
			destination: archive + "partial/" + &get_pkg_name(version),
			hash_type,
			hash_value,
			filename: version
				.get_record(RecordField::Filename)
				.expect("Record does not contain a filename!")
				.split_terminator('/')
				.last()
				.expect("Filename is malformed!")
				.to_string(),
			progress,
			is_finished: false,
			crash: false,
			errors: vec![],
		})))
	}
}

/// Struct used for sending progress data to UI
struct SubBar {
	is_total: bool,
	first_column: String,
	percentage: String,
	current_total: String,
	bytes_per_sec: String,
	ratio: f64,
}

impl SubBar {
	/// Consume the Bar and render it to the terminal.
	fn render(self, f: &mut Frame, chunk: Rc<[Rect]>) {
		f.render_widget(
			LineGauge::default()
				.line_set(symbols::line::THICK)
				.ratio(self.ratio)
				.label(self.first_column)
				.style(Style::default().fg(Color::White))
				.gauge_style(Style::default().fg(Color::Cyan).bg(Color::Red)),
			chunk[0],
		);
		f.render_widget(get_paragraph(&self.percentage), chunk[1]);
		f.render_widget(get_paragraph(&self.current_total), chunk[2]);
		f.render_widget(get_paragraph(&self.bytes_per_sec), chunk[3]);
	}
}

/// Struct used for aligning the progress segments
struct BarAlignment {
	pkg_name: usize,
	bar: u16,
	len: usize,
	current_total: usize,
	bytes_per_sec: usize,
	percentage: usize,
}

impl BarAlignment {
	fn new() -> Self {
		BarAlignment {
			pkg_name: 0,
			bar: 1024,
			len: 0,
			current_total: 0,
			bytes_per_sec: 0,
			percentage: 0,
		}
	}

	fn update_from_uri(&mut self, uri: MutexGuard<Uri>) {
		if uri.filename.len() > self.pkg_name {
			self.pkg_name = uri.filename.len()
		}

		if uri.progress.bar_length() < self.bar {
			self.bar = uri.progress.bar_length();
		}

		if uri.progress.current_total.len() > self.current_total {
			self.current_total = uri.progress.current_total.len();
		}

		if uri.progress.percentage.len() > self.percentage {
			self.percentage = uri.progress.percentage.len();
		}

		if uri.progress.bytes_per_sec.len() > self.bytes_per_sec {
			self.bytes_per_sec = uri.progress.bytes_per_sec.len();
		}

		// Increase the amount of downloads
		self.len += 1;
	}

	fn constraints(&self) -> [Constraint; 4] {
		[
			Constraint::Length(self.bar),
			Constraint::Length(self.percentage as u16 + 2),
			Constraint::Length(self.current_total as u16 + 2),
			Constraint::Length(self.bytes_per_sec as u16 + 2),
		]
	}
}

pub struct Progress {
	indicatif: ProgressBar,
	bytes_per_sec: String,
	current_total: String,
	percentage: String,
}

impl Progress {
	fn new(total: u64) -> Self {
		let indicatif = ProgressBar::hidden();
		indicatif.set_length(total);

		Progress {
			indicatif,
			bytes_per_sec: String::new(),
			current_total: String::new(),
			percentage: String::new(),
		}
	}

	fn ratio(&self) -> f64 {
		self.indicatif.position() as f64 / self.indicatif.length().unwrap() as f64
	}

	fn update_strings(&mut self) {
		self.bytes_per_sec = format!(
			"{}/s",
			unit_str(self.indicatif.per_sec() as u64, NumSys::Binary)
		);
		self.current_total = format!(
			"{}/{}",
			unit_str(self.indicatif.position(), NumSys::Binary),
			unit_str(self.indicatif.length().unwrap(), NumSys::Binary)
		);
		self.percentage = format!("{:.1} %", self.ratio() * 100.0);
	}

	fn bytes_per_sec(&self) -> &str { &self.bytes_per_sec }

	fn current_total(&self) -> &str { &self.current_total }

	fn percentage(&self) -> &str { &self.percentage }

	fn bar_length(&self) -> u16 {
		(terminal_width()
			- (self.percentage().len()
				+ self.current_total().len()
				+ self.bytes_per_sec().len()
				+ 8)) as u16
	}
}

pub struct Downloader {
	uri_list: Vec<Arc<Mutex<Uri>>>,
	untrusted: HashSet<String>,
	not_found: Vec<String>,
	mirrors: HashMap<String, String>,
	mirror_regex: MirrorRegex,
	progress: Arc<Mutex<Progress>>,
}

impl Downloader {
	fn new() -> Self {
		Downloader {
			uri_list: vec![],
			untrusted: HashSet::new(),
			not_found: vec![],
			mirrors: HashMap::new(),
			mirror_regex: MirrorRegex::new(),
			progress: Arc::new(Mutex::new(Progress::new(0))),
		}
	}

	async fn total_uris_finished(&self) -> usize {
		let mut total = 0;
		for uri in self.uri_list.iter() {
			if uri.lock().await.is_finished {
				total += 1;
			}
		}
		total
	}

	/// Return a Vector of contraints for ratatui
	async fn get_contraints(&self) -> Vec<Constraint> {
		let unlocked = self.progress.lock().await;

		vec![
			Constraint::Length(unlocked.bar_length() - 2),
			Constraint::Length(unlocked.percentage().len() as u16 + 2),
			Constraint::Length(unlocked.current_total().len() as u16 + 2),
			Constraint::Length(unlocked.bytes_per_sec().len() as u16 + 2),
		]
	}

	/// Generate the uri progress bars for packages.
	async fn gen_uri_bars(&self, align: &BarAlignment) -> Vec<SubBar> {
		let mut sub_bars = vec![];
		for uri in &self.uri_list {
			let unlocked = uri.lock().await;

			// Don't add finished downloads
			if unlocked.is_finished {
				continue;
			}

			sub_bars.push(SubBar {
				is_total: false,
				// Pad the package name if necessary for alignment.
				first_column: unlocked.filename.to_string()
					+ &" ".repeat(align.pkg_name - unlocked.filename.len()),
				ratio: unlocked.progress.ratio(),
				percentage: unlocked.progress.percentage().to_string(),
				current_total: unlocked.progress.current_total().to_string(),
				bytes_per_sec: unlocked.progress.bytes_per_sec().to_string(),
			})
		}

		// Generate total progress bar and put it last in the vector
		let unlocked = self.progress.lock().await;
		sub_bars.push(SubBar {
			is_total: true,
			first_column: format!(
				"Time Remaining: {}",
				FormattedDuration(unlocked.indicatif.eta())
			),
			percentage: unlocked.percentage().to_string(),
			current_total: unlocked.current_total().to_string(),
			bytes_per_sec: unlocked.bytes_per_sec().to_string(),
			ratio: unlocked.ratio(),
		});
		sub_bars
	}

	/// Calculate the text alignment for rendering
	async fn calculate_alignment(&mut self) -> BarAlignment {
		let mut align = BarAlignment::new();
		for uri in self.uri_list.iter_mut() {
			let mut unlocked = uri.lock().await;

			// Don't calculate finished downloads
			if unlocked.is_finished {
				continue;
			}
			unlocked.progress.update_strings();

			align.update_from_uri(unlocked);
		}
		align
	}

	/// Set the total for total progress based on the totals for Uri Progress.
	async fn set_total(&mut self) {
		let mut total = 0;
		for uri in &self.uri_list {
			total += uri.lock().await.progress.indicatif.length().unwrap();
		}
		self.progress.lock().await.indicatif.set_length(total);
	}

	/// Add the filtered Uris into the HashSet if applicable.
	fn get_from_mirrors<'a>(
		&self,
		version: &'a Version<'a>,
		uris: &mut Vec<String>,
		filename: &str,
	) -> Result<bool> {
		if let Some(data) = self.mirrors.get(filename) {
			for line in data.lines() {
				if !line.is_empty() && !line.starts_with('#') {
					uris.push(
						line.to_string()
							+ "/" + &version.get_record(RecordField::Filename).unwrap(),
					);
				}
			}
			return Ok(true);
		}
		Ok(false)
	}

	async fn add_to_mirrors(&mut self, uri: &str, filename: &str) -> Result<()> {
		self.mirrors.insert(
			filename.to_string(),
			match uri.starts_with("mirror+file:") {
				true => std::fs::read_to_string(filename)
					.with_context(|| format!("Failed to read {filename}, using defaults"))?,
				false => {
					reqwest::get("http://".to_string() + filename)
						.await?
						.text()
						.await?
				},
			},
		);
		Ok(())
	}

	/// Filter Uris from a package version.
	/// This will normalize different kinds of possible Uris
	/// Which are not http
	async fn filter_uris<'a>(
		&mut self,
		version: &'a Version<'a>,
		cache: &Cache,
		config: &Config,
	) -> Result<Vec<String>> {
		let mut filtered = Vec::new();

		for uri in version.uris() {
			// Sending a file path through the downloader will cause it to lock up
			// These have already been handled before the downloader runs.
			// TODO: We haven't actually handled anything yet. In python nala it happens
			// before it gets here. lol
			if uri.starts_with("file:") {
				continue;
			}

			if !uri_trusted(cache, version)? {
				self.untrusted
					.insert(config.color.red(version.parent().name()).to_string());
			}

			// We should probably consolidate this. And maybe test if mirror: works.
			if uri.starts_with("mirror+file:") || uri.starts_with("mirror:") {
				if let Some(file_match) = self.mirror_regex.mirror()?.captures(&uri) {
					let filename = file_match.get(1).unwrap().as_str();
					if !self.mirrors.contains_key(filename) {
						self.add_to_mirrors(&uri, filename).await?;
					};

					if self.get_from_mirrors(version, &mut filtered, filename)? {
						continue;
					}
				}

				if let Some(file_match) = self.mirror_regex.mirror_file()?.captures(&uri) {
					let filename = file_match.get(1).unwrap().as_str();
					if !self.mirrors.contains_key(filename) {
						self.add_to_mirrors(&uri, filename).await?;
					};

					if self.get_from_mirrors(version, &mut filtered, filename)? {
						continue;
					}
				}
			}

			// If none of the conditions meet then we just add it to the uris
			filtered.push(uri);
		}
		Ok(filtered)
	}
}

pub fn uri_trusted<'a>(cache: &Cache, version: &'a Version<'a>) -> Result<bool> {
	for mut pf in version.package_files() {
		// TODO: There is a bug here with codium specifically
		// It doesn't have an archive. Check apt source for clues here.
		if pf.archive()? != "now" {
			return Ok(cache.is_trusted(&mut pf));
		}
	}
	Ok(false)
}

pub fn untrusted_error(config: &Config, untrusted: &HashSet<String>) -> Result<()> {
	config
		.color
		.warn("The Following packages cannot be authenticated!");

	eprintln!(
		"  {}",
		untrusted
			.iter()
			.map(|s| s.to_string())
			.collect::<Vec<String>>()
			.join(", ")
	);

	if !config.apt.bool("APT::Get::AllowUnauthenticated", false) {
		bail!("Some packages were unable to be authenticated.")
	}

	config
		.color
		.notice("Configuration is set to allow installation of unauthenticated packages.");
	Ok(())
}

#[tokio::main]
pub async fn download(config: &Config) -> Result<()> {
	let mut downloader = Downloader::new();

	if let Some(pkg_names) = config.pkg_names() {
		// Dedupe the pkg names. If the same pkg is given twice
		// it will be downloaded twice, and then fail when moving the file
		let mut deduped = pkg_names.clone();
		deduped.sort();
		deduped.dedup();

		// Create the partial directory
		fs::create_dir("./partial").await?;

		let cache = new_cache!()?;
		for name in &deduped {
			if let Some(pkg) = cache.get(name) {
				let versions: Vec<Version> = pkg.versions().collect();
				for version in &versions {
					if version.is_downloadable() {
						let uri =
							// Download command defaults to current directory
							Uri::from_version(version, &cache, config, &mut downloader, "./".to_string()).await?;
						downloader.uri_list.push(uri);
						break;
					}
					// Version wasn't downloadable
					config.color.warn(&format!(
						"Can't find a source to download version '{}' of '{}'",
						version.version(),
						pkg.fullname(false)
					));
				}
			} else {
				downloader
					.not_found
					.push(config.color.yellow(name).to_string());
			}
		}
	} else {
		bail!("You must specify a package")
	};

	if !downloader.not_found.is_empty() {
		for pkg in &downloader.not_found {
			config.color.error(&format!("{pkg} not found"))
		}
		bail!("Some packages were not found.");
	}

	if !downloader.untrusted.is_empty() {
		untrusted_error(config, &downloader.untrusted)?
	}

	// Must set the total from the Uris we just gathered.
	downloader.set_total().await;

	// Set up the futures
	let mut set = JoinSet::new();
	for uri in &downloader.uri_list {
		set.spawn(download_file(downloader.progress.clone(), uri.clone()));
	}

	// setup terminal
	enable_raw_mode()?;
	let mut stdout = std::io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	// let mut terminal = Terminal::new(backend)?;
	let mut terminal = Terminal::with_options(
		backend,
		TerminalOptions {
			viewport: Viewport::Fullscreen,
		},
	)?;

	// create app and run it
	let tick_rate = Duration::from_millis(250);
	let res = run_app(&mut terminal, &mut downloader, tick_rate).await;
	disable_raw(&mut terminal)?;

	// This is for closing out of the app.
	if res.is_ok() {
		set.abort_all();
	}

	// Run all of the futures.
	while let Some(res) = set.join_next().await {
		res??;
	}

	if res.is_err() {
		res?
	}

	println!("Downloads Complete:");
	for uri in &downloader.uri_list {
		let unlocked = uri.lock().await;
		// Don't print packages that are not finished
		if !unlocked.is_finished {
			continue;
		}
		println!(
			"  {} was written to {}",
			config.color.package(&unlocked.filename),
			config.color.package(&unlocked.archive),
		)
	}

	// Time to print any errors
	for uri in &downloader.uri_list {
		let unlocked = uri.lock().await;

		if !unlocked.is_finished {
			for error in &unlocked.errors {
				config.color.error(error);
			}
		}
	}

	// Finally remove the partial directory
	fs::remove_dir("./partial").await?;

	Ok(())
}

async fn run_app<B: Backend>(
	terminal: &mut Terminal<B>,
	downloader: &mut Downloader,
	tick_rate: Duration,
) -> std::io::Result<()> {
	let mut last_tick = Instant::now();

	loop {
		// Check if we need to leave UI due to an error
		for uri in &downloader.uri_list {
			if uri.lock().await.crash {
				return Ok(());
			}
		}

		// If there are no more URIs it's time to leave the UI
		if downloader.total_uris_finished().await == downloader.uri_list.len() {
			return Ok(());
		}

		// Calculate the alignment for rendering.
		let align = downloader.calculate_alignment().await;

		// Update total information
		downloader.progress.lock().await.update_strings();

		let bars = downloader.gen_uri_bars(&align).await;
		let constraints = downloader.get_contraints().await;
		let pkgs_finished = downloader.total_uris_finished().await;

		// Async things have to be done outside of the UI function
		terminal.draw(|f| ui(f, align, bars, constraints, pkgs_finished, downloader))?;

		let timeout = tick_rate
			.checked_sub(last_tick.elapsed())
			.unwrap_or_else(|| Duration::from_secs(0));

		if crossterm::event::poll(timeout)? {
			if let Event::Key(key) = event::read()? {
				if let KeyCode::Char('q') = key.code {
					return Ok(());
				}
			}
		}
		if last_tick.elapsed() >= tick_rate {
			// app.on_tick();
			// TODO: We could potentially only update info at the tick rate.
			last_tick = Instant::now();
		}
	}
}

fn ui(
	f: &mut Frame,
	align: BarAlignment,
	sub_bars: Vec<SubBar>,
	total_constraints: Vec<Constraint>,
	pkgs_finished: usize,
	downloader: &Downloader,
) {
	// Create the outer downloading block
	let outer_block = build_block("  Downloading...  ".reset().bold());
	let chunks = split_vertical(uri_constraints(align.len), outer_block.inner(f.size()));

	// This is where we build the "block" to render things inside.
	let total_block = build_block("  Total Progress...  ".reset().bold());

	// We now create the inner block for the total block
	let total_inner_block = split_vertical(
		[Constraint::Min(1), Constraint::Min(1)],
		total_block.inner(chunks[align.len]),
	);

	// Start rendering our blocks
	f.render_widget(outer_block, f.size());
	f.render_widget(total_block, chunks[align.len]);

	// Render the bars
	f.render_widget(
		Paragraph::new(format!(
			"Packages: {pkgs_finished}/{}",
			downloader.uri_list.len()
		))
		.wrap(Wrap { trim: true })
		.alignment(Alignment::Left)
		.set_style(Style::default().fg(Color::White)),
		total_inner_block[0],
	);

	// This portion renders the progress bars.
	// The total progress bar is last.
	for (i, bar) in sub_bars.into_iter().enumerate() {
		let new_chunk = match bar.is_total {
			true => split_horizontal(total_constraints.as_slice(), total_inner_block[1]),
			false => split_horizontal(align.constraints(), chunks[i]),
		};
		bar.render(f, new_chunk);
	}
}

/// Splits a block horizontally with your contraints
fn split_horizontal<T>(constraints: T, block: Rect) -> Rc<[Rect]>
where
	T: IntoIterator,
	T::Item: Into<Constraint>,
{
	Layout::default()
		.direction(Direction::Horizontal)
		.constraints(constraints)
		.split(block)
}

/// Splits a block vertically with your contraints
fn split_vertical<T>(constraints: T, block: Rect) -> Rc<[Rect]>
where
	T: IntoIterator,
	T::Item: Into<Constraint>,
{
	Layout::default()
		.flex(Flex::Legacy)
		.direction(Direction::Vertical)
		.constraints(constraints)
		.split(block)
}

/// Build constraints based on how many downloads
fn uri_constraints(num: usize) -> Vec<Constraint> {
	let mut constraints = vec![
		// Last Constraint stops element expansion
		Constraint::Min(0),
	];

	for _ in 0..num {
		constraints.insert(0, Constraint::Max(1));
	}
	constraints
}

fn get_paragraph(text: &str) -> Paragraph {
	Paragraph::new(text)
		.wrap(Wrap { trim: true })
		.alignment(Alignment::Right)
		.set_style(Style::default().fg(Color::White))
}

fn build_block<'a, T: Into<Title<'a>>>(title: T) -> Block<'a> {
	Block::new()
		.borders(Borders::ALL)
		.border_type(BorderType::Rounded)
		.title_alignment(Alignment::Center)
		.title(title)
		.style(
			Style::default()
				.fg(Color::Cyan)
				.add_modifier(Modifier::BOLD),
		)
}

/// Restore Terminal
fn disable_raw<B: std::io::Write + Backend>(terminal: &mut Terminal<B>) -> Result<()> {
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture,
	)?;
	terminal.show_cursor()?;
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
		config.color.yellow(version.parent().name()),
		config.color.yellow(version.version()),
	);
}

pub async fn download_file(progress: Arc<Mutex<Progress>>, uri: Arc<Mutex<Uri>>) -> Result<()> {
	let client = reqwest::Client::new();

	loop {
		// Break out of the loop if there are no URIs left
		if uri.lock().await.uris.is_empty() {
			// If the uris are empty something is wrong.
			uri.lock().await.crash = true;
			break;
		}

		let mut response = open_connection(&client, uri.clone()).await?;

		let mut hasher: Box<dyn DynDigest + Send> = match uri.lock().await.hash_type.as_str() {
			"sha256" => Box::new(Sha256::new()),
			_ => Box::new(Sha512::new()),
		};

		let dest = uri.lock().await.destination.to_string();
		let file = open_file(uri.clone(), &dest).await?;
		let mut writer = BufWriter::new(file);

		// Iter over the response stream and update the hasher and progress bars
		while let Some(chunk) = get_chunk(&mut response, uri.clone()).await? {
			progress.lock().await.indicatif.inc(chunk.len() as u64);
			uri.lock().await.progress.indicatif.inc(chunk.len() as u64);
			hasher.update(&chunk);
			writer.write_all(&chunk).await?;
		}

		let mut download_hash = String::new();
		for byte in hasher.finalize().as_ref() {
			write!(&mut download_hash, "{:02x}", byte).expect("Unable to write hash to string");
		}

		// Handle if the hash doesn't check out.
		if download_hash != uri.lock().await.hash_value {
			uri.lock().await.errors.push(format!(
				"Checksum did not match for {}",
				&uri.lock().await.filename
			));
			uri.lock().await.uris.remove(0);

			// Remove the bad file so that it can't be used at all.
			fs::remove_file(&dest).await?;
			continue;
		}

		// Move the good file from partial to the archive dir.
		move_file(uri.clone(), &dest).await?;

		// Mark the uri as done downloading
		uri.lock().await.is_finished = true;
		break;
	}
	Ok(())
}

async fn open_connection(client: &reqwest::Client, uri: Arc<Mutex<Uri>>) -> Result<Response> {
	let response_res = client
		// There should always be a uri in here
		.get(uri.lock().await.uris.first().unwrap())
		.send()
		.await;

	// If a URI is bad just log the error and try the next one.
	if let Err(err) = &response_res {
		uri.lock().await.errors.push(err.to_string());
		uri.lock().await.uris.remove(0);
	}

	Ok(response_res?)
}

async fn open_file(uri: Arc<Mutex<Uri>>, dest: &str) -> Result<File> {
	// Create the File to write the download into
	let file_res = fs::File::create(dest).await;

	// Handle error and crash if we can't write files
	if file_res.is_err() {
		uri.lock().await.crash = true;
	}
	file_res.with_context(|| format!("Could not create file '{dest}'"))
}

async fn move_file(uri: Arc<Mutex<Uri>>, dest: &str) -> Result<()> {
	let archive_dest = uri.lock().await.archive.to_string();

	let file_res = fs::rename(dest, &archive_dest).await;

	if file_res.is_err() {
		uri.lock().await.crash = true;
	}
	file_res.with_context(|| format!("Could not move '{dest}' to '{archive_dest}'"))
}

async fn get_chunk(response: &mut Response, uri: Arc<Mutex<Uri>>) -> Result<Option<Bytes>> {
	let chunk = response.chunk().await;

	if let Err(err) = &chunk {
		uri.lock().await.errors.push(err.to_string());
		uri.lock().await.uris.remove(0);
	}

	Ok(chunk?)
}
