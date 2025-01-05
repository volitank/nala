#[macro_export]
/// Print Debug information using NalaProgress.
macro_rules! dprog {
	($config:expr, $progress:expr, $context:expr, $(,)? $($arg:tt)*) => {
		if $config.debug() {
			let output = std::fmt::format(std::format_args!($($arg)*));
			if $progress.hidden() {
				eprintln!("DEBUG({}): {output}", $context);
			} else {
				$progress.print(&format!("DEBUG({}): {output}", $context))?;
			}
		}
	};
}

use std::sync::LazyLock;

use anyhow::{bail, Result};
use regex::{Regex, RegexBuilder};
use rust_apt::records::RecordField;
use rust_apt::Version;

use crate::config::{color, Config, Theme};

fn build_regex(regex: &str) -> Regex {
	RegexBuilder::new(regex)
		.case_insensitive(true)
		.build()
		.unwrap()
}

macro_rules! lazy_regex {
	($($name:ident => $re:literal),*) => {
		$(
			pub static $name: LazyLock<Regex> = LazyLock::new(|| build_regex($re));
		)*
	};
}

lazy_regex!(
	MIRROR => r"(mirror://(.*?)/pool|mirror\+file:(/.*?)/pool)",
	// Regex for formating the Apt sources from URI.
	URL => "(https?://.*?/.*?/)",
	// Regex for finding Pacstall remote repo
	PACSTALL => r#"_remoterepo="(.*?)""#,
	DOMAIN => r"https?://([A-Za-z_0-9.-]+).*",
	UBUNTU_URL => r"<link>(.*)</link>",
	UBUNTU_COUNTRY => r"<mirror:countrycode>(.*)</mirror:countrycode>"
);

pub fn version_diff(old: &str, new: String) -> String {
	// Check for just revision change first.
	if let (Some(old_ver), Some(new_ver)) = (old.rsplit_once('-'), new.rsplit_once('-')) {
		// If there isn't a revision these shouldn't ever match
		// If they do match then only the revision has changed
		if old_ver.0 == new_ver.0 {
			return format!("{}-{}", new_ver.0, color::color!(Theme::Notice, new_ver.0));
		}
	}

	let (old_ver, new_ver) = (
		old.split('.').collect::<Vec<_>>(),
		new.split('.').collect::<Vec<_>>(),
	);

	let mut start_color = 0;
	for (i, section) in old_ver.iter().enumerate() {
		if i > new_ver.len() - 1 {
			break;
		}

		if section != &new_ver[i] {
			start_color = i;
			break;
		}
	}

	new_ver
		.iter()
		.enumerate()
		.map(|(i, str)| {
			if i >= start_color {
				color::color!(Theme::Notice, str).to_string()
			} else {
				str.to_string()
			}
		})
		.collect::<Vec<_>>()
		.join(".")
}

/// Return the package name. Checks if epoch is needed.
pub fn get_pkg_name(version: &Version) -> String {
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

#[link(name = "c")]
extern "C" {
	pub fn geteuid() -> u32;
}

/// Check for root. Errors if not root.
/// Set up lock file if root.
pub fn sudo_check(config: &Config) -> Result<()> {
	if unsafe { geteuid() != 0 } {
		bail!("Nala needs root to {}", config.command)
	}
	// TODO: Need to add lock file logic here maybe.
	Ok(())
}

/// Get the username or return Unknown.
pub fn get_user() -> (std::string::String, std::string::String) {
	let uid = std::env::var("SUDO_UID").unwrap_or_else(|_| format!("{}", unsafe { geteuid() }));

	let username = std::env::var("SUDO_USER").unwrap_or_else(|_| {
		for key in ["LOGNAME", "USER", "LNAME", "USERNAME"] {
			if let Ok(name) = std::env::var(key) {
				return name;
			}
		}
		"Unknown".to_string()
	});

	(uid, username)
}
