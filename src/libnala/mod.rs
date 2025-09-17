mod cache;
mod history;
mod operation;
mod package;
mod shell;
mod show;
mod system;

use std::sync::LazyLock;

pub use cache::NalaCache;
pub use history::{get_history, HistoryEntry, HistoryFile, HistoryPackage};
pub use operation::Operation;
pub use package::{NalaPkg, NalaVersion};
use regex::{Regex, RegexBuilder};
pub use shell::{apt_hook_with_pkgs, run_scripts};
pub use show::ShowVersion;
pub use system::sudo_check;

macro_rules! lazy_regex {
	($($name:ident => $re:literal),*) => {
		$(
			pub static $name: LazyLock<Regex> = LazyLock::new(|| {
				RegexBuilder::new($re).case_insensitive(true).build().unwrap()
			});
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
