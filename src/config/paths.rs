const HISTORY_DEFAULT: &str = "/var/lib/nala/history";
const NALA_SOURCES_PATH: &str = "/etc/apt/sources.list.d/nala.sources";

#[derive(Clone, Copy)]
pub enum PathSpec {
	Apt {
		key: &'static str,
		default: &'static str,
	},
	Fixed(&'static str),
}

/// Represents different file and directory paths
pub enum Paths {
	/// The Archive dir holds packages.
	/// Default dir `/var/cache/apt/archives/`
	Archive,
	/// The Lists dir hold package lists from `update` command.
	/// Default dir `/var/lib/apt/lists/`
	Lists,
	/// The main Source List.
	/// Default file `/etc/apt/sources.list`
	SourceList,
	/// The Sources parts directory
	/// Default dir `/etc/apt/sources.list.d/`
	SourceParts,
	/// Nala Sources file is generated from the `fetch` command.
	/// Default file `/etc/apt/sources.list.d/nala-sources.list`
	NalaSources,

	History,
}

impl Paths {
	pub fn spec(&self) -> PathSpec {
		match self {
			Paths::Archive => PathSpec::Apt {
				key: "Dir::Cache::Archives",
				default: "/var/cache/apt/archives/",
			},
			Paths::Lists => PathSpec::Apt {
				key: "Dir::State::Lists",
				default: "/var/lib/apt/lists/",
			},
			Paths::SourceList => PathSpec::Apt {
				key: "Dir::Etc::sourcelist",
				default: "/etc/apt/sources.list",
			},
			Paths::SourceParts => PathSpec::Apt {
				key: "Dir::Etc::sourceparts",
				default: "/etc/apt/sources.list.d/",
			},
			Paths::NalaSources => PathSpec::Fixed(NALA_SOURCES_PATH),
			Paths::History => PathSpec::Fixed(HISTORY_DEFAULT),
		}
	}

	pub fn path(&self) -> &'static str {
		match self.spec() {
			PathSpec::Apt { key, .. } => key,
			PathSpec::Fixed(path) => path,
		}
	}

	pub fn default_path(&self) -> &'static str {
		match self.spec() {
			PathSpec::Apt { default, .. } => default,
			PathSpec::Fixed(path) => path,
		}
	}
}
