use std::sync::LazyLock;

use regex::{Regex, RegexBuilder};

fn build_regex(regex: &str) -> Regex {
	RegexBuilder::new(regex)
		.case_insensitive(true)
		.build()
		.unwrap()
}

macro_rules! lazy_regex {
	($($name:ident => $re:literal),* $(,)?) => {
		$(
			pub static $name: LazyLock<Regex> = LazyLock::new(|| build_regex($re));
		)*
	};
}

lazy_regex!(
	MIRROR => r"(mirror://(.*?)/pool|mirror\+file:(/.*?)/pool)",
	URL => "(https?://.*?/.*?/)",
	PACSTALL => r#"_remoterepo="(.*?)""#,
	DOMAIN => r"https?://([A-Za-z_0-9.-]+).*",
	UBUNTU_URL => r"<link>(.*)</link>",
	UBUNTU_COUNTRY => r"<mirror:countrycode>(.*)</mirror:countrycode>",
);
