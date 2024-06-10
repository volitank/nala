use std::time::Duration;

use anyhow::Result;
use crossterm::{cursor, execute};
use indicatif::{ProgressBar, ProgressStyle};
use rust_apt::error::{pending_error, AptErrors};
use rust_apt::progress::{AcquireProgress, DynAcquireProgress};
use rust_apt::raw::{AcqTextStatus, ItemDesc, ItemState, PkgAcquire};
use rust_apt::util::{terminal_width, time_str, unit_str, NumSys};
use rust_apt::{new_cache, PackageSort};

use crate::config::Config;

pub fn update(config: &Config) -> Result<(), AptErrors> {
	let cache = new_cache!()?;
	let mut stdout = std::io::stdout();

	// TODO: Handle CtrlC here so that we can unhide the cursor
	execute!(stdout, cursor::Hide)?;

	let res = cache.update(&mut AcquireProgress::new(NalaAcquireProgress::new(config)));

	execute!(stdout, cursor::Show)?;

	// Do not print how many packages are upgradable if update errored.
	#[allow(clippy::question_mark)]
	if res.is_err() {
		return res;
	}

	let cache = new_cache!()?;
	let sort = PackageSort::default().upgradable();
	let upgradable: Vec<_> = cache.packages(&sort).collect();

	if !upgradable.is_empty() {
		println!(
			"{} packages can be upgraded. Run '{}' to see them.",
			config.color.yellow(&format!("{}", upgradable.len())),
			config.color.package("nala list --upgradable")
		);
	}

	// Not sure yet if I want to implement this directly
	// But here is how one might do it.
	//
	// for pkg in upgradable {
	// 	let (Some(inst), Some(cand)) = (pkg.installed(), pkg.candidate()) else {
	// 		continue;
	// 	};

	// 	println!("{pkg} ({inst}) -> ({cand})");
	// }

	res
}

// "┏━┳┓\n"
// "┃ ┃┃\n"
// "┣━╋┫\n"
// "┃ ┃┃\n"
// "┣━╋┫\n"
// "┣━╋┫\n"
// "┃ ┃┃\n"
// "┗━┻┛\n"

// "╭─┬╮\n"
// "│ ││\n"
// "├─┼┤\n"
// "│ ││\n"
// "├─┼┤\n"
// "├─┼┤\n"
// "│ ││\n"
// "╰─┴╯\n"

const HEAVY: [&str; 6] = ["━", "┃", "┏", "┓", "┗", "┛"];
#[allow(dead_code)]
const ROUNDED: [&str; 6] = ["─", "│", "╭", "╮", "╰", "╯"];
#[allow(dead_code)]
const ASCII: [&str; 6] = ["-", "|", "x", "x", "x", "x"];

pub struct Border {
	chars: [&'static str; 6],
	width: usize,
	top: String,
	bot: String,
}

impl Border {
	pub fn new(chars: [&'static str; 6]) -> Border {
		Self {
			chars,
			width: 0,
			top: String::new(),
			bot: String::new(),
		}
	}

	pub fn horizontal(&self) -> &str { self.chars[0] }

	pub fn vertical(&self) -> &str { self.chars[1] }

	pub fn width_changed(&mut self) -> bool {
		let width = terminal_width();
		if self.width != width {
			self.width = width;
			return true;
		}
		false
	}

	pub fn top_border(&mut self) -> &str {
		if self.width_changed() || self.top.is_empty() {
			let header = "Update";

			self.top = format!(
				"{}{} {header} {}{}",
				self.chars[2],
				self.horizontal(),
				self.horizontal().repeat(self.width - header.len() - 5),
				self.chars[3],
			);
		}
		&self.top
	}

	pub fn bottom_border(&mut self) -> &str {
		if self.width_changed() || self.bot.is_empty() {
			self.bot = format!(
				"{}{}{}",
				self.chars[4],
				self.horizontal().repeat(self.width - 2),
				self.chars[5],
			)
		}
		&self.bot
	}
}

/// AptAcquireProgress is the default struct for the update method on the cache.
///
/// This struct mimics the output of `apt update`.
pub struct NalaAcquireProgress<'a> {
	config: &'a Config,
	pulse_interval: usize,
	max: usize,
	progress: ProgressBar,
	border: Border,
	ign: String,
	hit: String,
	get: String,
	err: String,
}

impl<'a> NalaAcquireProgress<'a> {
	/// Returns a new default progress instance.
	pub fn new(config: &'a Config) -> Self {
		let mut progress = Self {
			config,
			pulse_interval: 0,
			max: 0,
			progress: ProgressBar::hidden(),
			// TODO: Maybe we should make it configurable.
			border: Border::new(HEAVY),
			ign: config.color.yellow("Ignored").into(),
			hit: config.color.package("No Change").into(),
			get: config.color.blue("Updated").into(),
			err: config.color.red("Error").into(),
		};
		progress.progress = ProgressBar::new(0)
			.with_style(progress.get_style("".to_string()))
			.with_prefix("%");

		progress.progress.enable_steady_tick(Duration::from_secs(1));
		progress
	}

	pub fn get_style(&mut self, msg: String) -> ProgressStyle {
		// "┏━┳┓\n"
		// "┃ ┃┃\n"
		// "┣━╋┫\n"
		// "┃ ┃┃\n"
		// "┣━╋┫\n"
		// "┣━╋┫\n"
		// "┃ ┃┃\n"
		// "┗━┻┛\n"

		// "╭─┬╮\n"
		// "│ ││\n"
		// "├─┼┤\n"
		// "│ ││\n"
		// "├─┼┤\n"
		// "├─┼┤\n"
		// "│ ││\n"
		// "╰─┴╯\n"

		// TODO: This will need to be methods on the struct
		// And we will have to shift it around to be ASCII safe maybe.
		let mut template = self
			.config
			.color
			.package(self.border.top_border())
			.to_string();

		template += "\n";
		if !msg.is_empty() {
			template += &msg;
			template += "\n";
		}

		let side = self
			.config
			.color
			.package(self.border.vertical())
			.to_string();

		template += &format!(
			"{side} {{spinner:.bold}} {{percent:.bold}}{{prefix:.bold}} [{{wide_bar:.cyan/red}}] \
			 {{bytes}}/{{total_bytes}} ({{eta}}) {side}\n{}",
			self.config.color.package(self.border.bottom_border())
		);

		ProgressStyle::with_template(&template)
			.unwrap()
			.tick_strings(&[".  ", ".. ", "...", "   "])
			.progress_chars("━━")
	}
}

impl<'a> DynAcquireProgress for NalaAcquireProgress<'a> {
	/// Used to send the pulse interval to the apt progress class.
	///
	/// Pulse Interval is in microseconds.
	///
	/// Example: 1 second = 1000000 microseconds.
	///
	/// Apt default is 500000 microseconds or 0.5 seconds.
	///
	/// The higher the number, the less frequent pulse updates will be.
	///
	/// Pulse Interval set to 0 assumes the apt defaults.
	fn pulse_interval(&self) -> usize { self.pulse_interval }

	/// Called when an item is confirmed to be up-to-date.
	///
	/// Prints out the short description and the expected size.
	fn hit(&mut self, item: &ItemDesc) {
		self.progress
			.println(format!("{}: {}", self.hit, item.description()));
	}

	/// Called when an Item has started to download
	///
	/// Prints out the short description and the expected size.
	fn fetch(&mut self, item: &ItemDesc) {
		let mut msg = format!("{}:   {}", self.get, item.description());

		let file_size = item.owner().file_size();
		if file_size != 0 {
			msg += &format!(" [{}]", unit_str(file_size, NumSys::Decimal))
		}

		self.progress.println(msg);
	}

	/// Called when an item is successfully and completely fetched.
	///
	/// We don't print anything here to remain consistent with apt.
	fn done(&mut self, _: &ItemDesc) {}

	/// Called when progress has started.
	///
	/// Start does not pass information into the method.
	///
	/// We do not print anything here to remain consistent with apt.
	fn start(&mut self) {}

	/// Called when progress has finished.
	///
	/// Stop does not pass information into the method.
	///
	/// prints out the bytes downloaded and the overall average line speed.
	fn stop(&mut self, status: &AcqTextStatus) {
		if pending_error() {
			return;
		}

		let msg = if status.fetched_bytes() != 0 {
			self.config
				.color
				.bold(&format!(
					"Fetched {} in {} ({}/s)",
					unit_str(status.fetched_bytes(), NumSys::Decimal),
					time_str(status.elapsed_time()),
					unit_str(status.current_cps(), NumSys::Decimal)
				))
				.to_string()
		} else {
			"Nothing to fetch.".to_string()
		};
		self.progress.println(msg);
	}

	/// Called when an Item fails to download.
	///
	/// Print out the ErrorText for the Item.
	fn fail(&mut self, item: &ItemDesc) {
		let mut show_error = self
			.config
			.apt
			.bool("Acquire::Progress::Ignore::ShowErrorText", true);
		let error_text = item.owner().error_text();

		let header = match item.owner().status() {
			ItemState::StatIdle | ItemState::StatDone => {
				if error_text.is_empty() {
					show_error = false;
				}
				&self.ign
			},
			_ => &self.err,
		};

		self.progress
			.println(format!("{header}: {}", item.description()));

		if show_error {
			self.progress.println(error_text);
		}
	}

	/// Called periodically to provide the overall progress information
	///
	/// Draws the current progress.
	/// Each line has an overall percent meter and a per active item status
	/// meter along with an overall bandwidth and ETA indicator.
	fn pulse(&mut self, status: &AcqTextStatus, owner: &PkgAcquire) {
		self.progress.set_length(status.total_bytes());
		self.progress.set_position(status.current_bytes());

		let term_width = terminal_width();
		let side = self.config.color.package(self.border.vertical());
		let mut string: Vec<String> = vec![];

		for worker in owner.workers().iter() {
			let Ok(item) = worker.item() else {
				continue;
			};

			let owner = item.owner();
			if owner.status() != ItemState::StatFetching {
				continue;
			}

			let mut work_string = owner.active_subprocess();
			if work_string.is_empty() {
				work_string += "Downloading"
			} else if work_string == "store" {
				work_string = "Processing".to_string()
			}
			work_string += ": ";

			if let Some(dest_file) = owner.dest_file().split_terminator('/').last() {
				// Decide on protocol.
				let proto = if item.uri().starts_with("https") { "https://" } else { "http://" };
				// Build the correct URI by destination file. item.uri() returns the /by-hash
				// link.
				let uri = dest_file.replace('_', "/");
				// Calculate string padding. - 2 for border. - 2 for spaces.
				let padding = term_width - work_string.len() - proto.len() - uri.len() - 4;

				string.push(format!(
					"{side} {} {proto}{uri}{}{side}",
					self.config.color.bold(&work_string),
					&" ".repeat(padding),
				));
				// Break only for slim progress
				if !self.config.get_bool("verbose", false) {
					break;
				}
			};
		}

		// If there are no worker strings just return
		if string.is_empty() {
			return;
		}

		self.max = string.len().max(self.max);

		let filler = format!("{side}{}{side}", &" ".repeat(term_width - 2));

		while string.len() < self.max {
			string.insert(0, filler.to_string())
		}

		let style = self.get_style(string.join("\n"));
		self.progress.set_style(style);
	}
}
