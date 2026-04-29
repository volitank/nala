use std::path::PathBuf;
use std::str::FromStr;

use clap::{ArgGroup, Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(name = "nala")]
#[clap(author = "Blake Lee <blake@volian.org>")]
#[clap(version = "0.1.0")]
#[clap(about = "Commandline front-end for libapt-pkg", long_about = None)]
pub struct NalaParser {
	/// Print license information
	#[clap(global = true, short, long, action)]
	pub license: bool,

	/// Disable scrolling text and print extra information
	#[clap(global = true, short, long, action)]
	pub verbose: bool,
	/// Print debug statements for solving issues
	#[clap(global = true, short, long, action)]
	pub debug: bool,

	/// Specify a different configuration file
	#[clap(short, long, value_parser, value_name = "FILE")]
	pub config: Option<PathBuf>,

	/// Override the history directory for tests and isolated fixtures
	#[clap(global = true, long, hide = true, value_name = "DIR")]
	pub history_dir: Option<PathBuf>,

	/// Turn on tui if it's disabled in the config.
	#[clap(global = true, long, action)]
	pub tui: bool,

	/// Turn the tui off. Takes precedence over other options
	#[clap(global = true, long, action)]
	pub no_tui: bool,

	/// Only download packages.
	#[clap(global = true, long, action)]
	pub download_only: bool,

	/// Display a simpler and more condensed transaction summary.
	#[clap(global = true, long, action)]
	pub simple: bool,

	/// Update package lists before running the command.
	#[clap(global = true, long, action, conflicts_with = "no_update")]
	pub update: bool,

	/// Do NOT update package lists before running the command.
	#[clap(global = true, long, action, conflicts_with = "update")]
	pub no_update: bool,

	/// Passthrough Apt configurations
	#[clap(global = true, short = 'o', long, action)]
	pub option: Vec<String>,

	/// Allow Nala to install packages that can't be hashsum verified
	#[clap(global = true, long, action)]
	pub allow_unauthenticated: bool,

	/// Install recommended packages.
	#[clap(global = true, long, action, conflicts_with = "no_install_recommends")]
	pub install_recommends: bool,

	/// Do NOT install recommended packages.
	#[clap(global = true, long, action, conflicts_with = "install_recommends")]
	pub no_install_recommends: bool,

	/// Install suggested packages.
	#[clap(global = true, long, action, conflicts_with = "no_install_suggests")]
	pub install_suggests: bool,

	/// Do NOT install suggested packages.
	#[clap(global = true, long, action, conflicts_with = "install_suggests")]
	pub no_install_suggests: bool,

	/// Set the default release to install packages from.
	#[clap(global = true, short = 't', long, value_name = "RELEASE")]
	pub target_release: Option<String>,

	/// Try to fix broken packages.
	#[clap(
		global = true,
		short = 'f',
		long,
		action,
		conflicts_with = "no_fix_broken"
	)]
	pub fix_broken: bool,

	/// Do NOT try to fix broken packages.
	#[clap(global = true, long, action, conflicts_with = "fix_broken")]
	pub no_fix_broken: bool,

	/// Assume yes for all prompts.
	#[clap(global = true, short = 'y', long, action, conflicts_with = "assume_no")]
	pub assume_yes: bool,

	/// Assume no for all prompts.
	#[clap(
		global = true,
		short = 'n',
		long,
		action,
		conflicts_with = "assume_yes"
	)]
	pub assume_no: bool,

	/// Additionally remove unnecessary packages.
	#[clap(
		global = true,
		long,
		visible_alias = "autoremove",
		action,
		conflicts_with = "no_auto_remove"
	)]
	pub auto_remove: bool,

	/// Do NOT remove unnecessary packages.
	#[clap(
		global = true,
		long,
		visible_alias = "no-autoremove",
		action,
		conflicts_with = "auto_remove"
	)]
	pub no_auto_remove: bool,

	/// Allow the removal of essential packages.
	#[clap(global = true, long, action)]
	pub remove_essential: bool,

	/// Remove config files for any package set to be removed.
	#[clap(global = true, long, action)]
	pub purge: bool,

	#[clap(subcommand)]
	pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "lower")]
pub enum Commands {
	List(List),
	Search(Search),
	Show(Show),
	Policy(Policy),
	Clean(Clean),
	Download(Download),
	History(History),
	Fetch(Fetch),
	Update(Update),
	Upgrade(Upgrade),
	Install(Install),
	Remove(Remove),
	AutoRemove(AutoRemove),
}

/// List all packages or only packages based on the provided name
#[derive(Args, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct List {
	/// Package names to search
	#[clap(required = false)]
	pub pkg_names: Vec<String>,

	/// Print the full description of each package
	#[clap(long, action)]
	pub description: bool,

	/// Print the summary of each package
	#[clap(long, action)]
	pub summary: bool,

	/// Show all versions of a package
	#[clap(short, long, action)]
	pub all_versions: bool,

	/// Only include packages that are installed
	#[clap(short, long, action)]
	pub installed: bool,

	/// Only include packages explicitly installed with Nala
	#[clap(short = 'N', long, action)]
	pub nala_installed: bool,

	/// Only include packages that are upgradable
	#[clap(short, long, action)]
	pub upgradable: bool,

	/// Only include virtual packages
	#[clap(short = 'V', long, action)]
	pub r#virtual: bool,

	#[clap(short = 'm', long, action)]
	pub machine: bool,
}

/// Like `List`, but uses regex and searches package descriptions.
#[derive(Args, Debug)]
pub struct Search {
	/// Search only using pkg names and not descriptions.
	#[clap(long, action)]
	pub names_only: bool,

	// Flatten list commands args into search
	#[clap(flatten)]
	pub list_args: List,
}

/// Show information about one or more packages
#[derive(Args, Debug)]
pub struct Show {
	/// Package names to show
	#[clap(required = false)]
	pub pkg_names: Vec<String>,

	#[clap(short = 'a', long, action)]
	pub all_versions: bool,

	#[clap(short = 'm', long, action)]
	pub machine: bool,
}

/// Show pin/priority information about one or more packages
#[derive(Args, Debug)]
pub struct Policy {
	/// Package names to show policy for
	#[clap(required = false)]
	pub pkg_names: Vec<String>,

	#[clap(short = 'm', long, action)]
	pub machine: bool,
}

/// Removes the local archive of downloaded package files.
#[derive(Args, Debug)]
pub struct Clean {
	/// Removes the package lists downloaded from `update`
	#[clap(long, action)]
	pub lists: bool,

	/// Removes the `nala-sources.list` file generated by the fetch command
	#[clap(long, action)]
	pub fetch: bool,
}

/// Downloads a package to the current directory.
#[derive(Args, Debug)]
pub struct Download {
	/// Package names to download
	pub pkg_names: Vec<String>,

	/// Removes the `nala-sources.list` file generated by the fetch command
	#[clap(long, action)]
	pub fetch: bool,
}

/// View or replay stored package transaction history.
#[derive(Args, Debug)]
#[clap(args_conflicts_with_subcommands = true)]
pub struct History {
	/// Show details for a specific history entry ID or `last`
	#[clap(value_name = "ID|last")]
	pub history_id: Option<HistorySelector>,

	/// Run a history action instead of showing list/detail output
	#[clap(subcommand)]
	pub command: Option<HistoryCommand>,
}

/// Additional actions supported by the history command.
#[derive(Subcommand, Debug)]
#[clap(rename_all = "lower")]
pub enum HistoryCommand {
	Undo(HistoryUndo),
	Redo(HistoryRedo),
	Clear(HistoryClear),
}

/// Replay the inverse of a previously applied history entry.
#[derive(Args, Debug)]
pub struct HistoryUndo {
	/// History entry ID or `last` to undo
	#[clap(value_name = "ID|last")]
	pub history_id: HistorySelector,
}

/// Replay the stored package actions from a previously applied history entry.
#[derive(Args, Debug)]
pub struct HistoryRedo {
	/// History entry ID or `last` to redo
	#[clap(value_name = "ID|last")]
	pub history_id: HistorySelector,
}

/// Clear a specific stored history entry or the entire history.
#[derive(Args, Debug)]
#[clap(group(
	ArgGroup::new("target")
		.required(true)
		.args(["history_id", "all"])
))]
pub struct HistoryClear {
	/// History entry ID or `last` to clear
	#[clap(value_name = "ID|last", conflicts_with = "all")]
	pub history_id: Option<HistorySelector>,

	/// Clear the entire stored history
	#[clap(long, action)]
	pub all: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HistorySelector {
	Last,
	Id(u32),
}

impl FromStr for HistorySelector {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		if value.eq_ignore_ascii_case("last") {
			return Ok(Self::Last);
		}

		value.parse::<u32>().map(Self::Id).map_err(|_| {
			format!("Invalid history selector '{value}'. Use an integer ID or 'last'.")
		})
	}
}

#[derive(Args, Debug)]
pub struct Fetch {
	#[clap(long, action)]
	pub non_free: bool,

	#[clap(long, action)]
	pub https_only: bool,

	#[clap(long, action)]
	pub sources: bool,

	#[clap(long, num_args = 0..=1, default_missing_value="3")]
	pub auto: Option<u8>,

	#[clap(short = 'c', long, action)]
	pub country: Vec<String>,

	#[clap(long, action)]
	pub debian: Option<String>,

	#[clap(long, action)]
	pub ubuntu: Option<String>,

	#[clap(long, action)]
	pub devuan: Option<String>,
}

/// Update the package lists.
#[derive(Args, Debug)]
pub struct Update {}

/// Upgrade packages.
#[derive(Args, Debug)]
#[clap(visible_aliases = ["full-upgrade", "safe-upgrade"])]
pub struct Upgrade {
	/// Prints the URIs in json and does not perform an upgrade.
	#[clap(long, action)]
	pub print_uris: bool,

	/// Exclude packages from upgrade. Accepts glob patterns.
	#[clap(long, value_name = "PKG", action)]
	pub exclude: Vec<String>,

	/// Perform a Full Upgrade.
	#[clap(long, action)]
	pub full: bool,

	/// Do NOT perform a Full Upgrade.
	#[clap(long, action)]
	pub no_full: bool,

	/// Perform a Safe Upgrade.
	/// Takes precedence over other Upgrade options.
	#[clap(long, action)]
	pub safe: bool,
}

#[derive(Args, Debug)]
/// Install Packages
pub struct Install {
	/// Package names to install
	#[clap(required = false)]
	pub pkg_names: Vec<String>,

	/// Reinstall packages that are already installed
	#[clap(long, action)]
	pub reinstall: bool,
}

#[derive(Args, Debug)]
#[clap(visible_alias = "purge")]
/// Remove Packages
///
/// Using the alias `purge` is the same as running
/// `nala remove --purge`
pub struct Remove {
	/// Package names to install
	pub pkg_names: Vec<String>,
}

#[derive(Args, Debug)]
#[clap(visible_alias = "autopurge")]
/// Automatically remove unnecessary packages
///
/// Using the alias `autopurge` is the same as running
/// `nala autoremove --purge`
pub struct AutoRemove {
	/// Additionally, when purging, remove pkgs in config state
	#[clap(long, action)]
	pub remove_config: bool,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn install_reinstall_flag_parses() {
		let parsed =
			NalaParser::try_parse_from(["nala", "install", "--reinstall", "demo"]).unwrap();

		let Some(Commands::Install(args)) = parsed.command else {
			panic!("expected install command");
		};

		assert!(args.reinstall);
		assert_eq!(args.pkg_names, vec!["demo"]);
	}

	#[test]
	fn assume_prompt_flags_parse() {
		let yes = NalaParser::try_parse_from(["nala", "install", "-y", "demo"]).unwrap();
		assert!(yes.assume_yes);
		assert!(!yes.assume_no);

		let no = NalaParser::try_parse_from(["nala", "install", "-n", "demo"]).unwrap();
		assert!(no.assume_no);
		assert!(!no.assume_yes);

		assert!(NalaParser::try_parse_from([
			"nala",
			"install",
			"--assume-yes",
			"--assume-no",
			"demo",
		])
		.is_err());
	}

	#[test]
	fn apt_behavior_flags_parse() {
		let parsed = NalaParser::try_parse_from([
			"nala",
			"install",
			"--install-recommends",
			"--no-install-suggests",
			"-t",
			"testing",
			"--no-fix-broken",
			"demo",
		])
		.unwrap();

		assert!(parsed.install_recommends);
		assert!(parsed.no_install_suggests);
		assert_eq!(parsed.target_release.as_deref(), Some("testing"));
		assert!(parsed.no_fix_broken);

		assert!(NalaParser::try_parse_from([
			"nala",
			"install",
			"--install-suggests",
			"--no-install-suggests",
			"demo",
		])
		.is_err());
	}

	#[test]
	fn transaction_safety_flags_parse() {
		let parsed = NalaParser::try_parse_from([
			"nala",
			"remove",
			"--remove-essential",
			"--no-autoremove",
			"demo",
		])
		.unwrap();

		assert!(parsed.remove_essential);
		assert!(parsed.no_auto_remove);

		let alias =
			NalaParser::try_parse_from(["nala", "install", "--autoremove", "demo"]).unwrap();
		assert!(alias.auto_remove);

		assert!(NalaParser::try_parse_from([
			"nala",
			"install",
			"--auto-remove",
			"--no-autoremove",
			"demo",
		])
		.is_err());
	}

	#[test]
	fn update_flags_parse() {
		let update = NalaParser::try_parse_from(["nala", "install", "--update", "demo"]).unwrap();
		assert!(update.update);
		assert!(!update.no_update);

		let no_update = NalaParser::try_parse_from(["nala", "upgrade", "--no-update"]).unwrap();
		assert!(no_update.no_update);
		assert!(!no_update.update);

		assert!(
			NalaParser::try_parse_from(["nala", "upgrade", "--update", "--no-update",]).is_err()
		);
	}

	#[test]
	fn upgrade_exclude_flags_parse() {
		let parsed = NalaParser::try_parse_from([
			"nala",
			"upgrade",
			"--exclude",
			"foo",
			"--exclude",
			"linux-*",
		])
		.unwrap();

		let Some(Commands::Upgrade(args)) = parsed.command else {
			panic!("expected upgrade command");
		};

		assert_eq!(args.exclude, vec!["foo", "linux-*"]);
	}

	#[test]
	fn simple_summary_flags_parse() {
		let simple = NalaParser::try_parse_from(["nala", "install", "--simple", "demo"]).unwrap();
		assert!(simple.simple);
	}
}
