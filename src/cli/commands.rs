use std::str::FromStr;

use clap::{ArgGroup, Args, Subcommand};
use clap_complete::engine::ArgValueCompleter;

use super::completion::{history_id_completion, installed_package_completion, package_completion};
use super::flags::{AutoRemoveFlags, FixBrokenFlags, InfoFlags, InstallFlags, TransactionFlags};
use crate::t;

/// All supported nala subcommands.
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
	#[clap(hide = true, disable_help_flag = true)]
	Moo(Moo),
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-list"))]
#[allow(clippy::struct_excessive_bools)]
pub struct List {
	#[clap(
		required = false,
		add = ArgValueCompleter::new(package_completion),
		help = t!("cli-pkg-search")
	)]
	pub pkg_names: Vec<String>,

	#[clap(long, action, help = t!("cli-description"))]
	pub description: bool,

	#[clap(long, action, help = t!("cli-summary"))]
	pub summary: bool,

	#[clap(flatten)]
	pub info: InfoFlags,

	#[clap(short, long, action, help = t!("cli-installed"))]
	pub installed: bool,

	#[clap(
		short = 'N',
		long,
		action,
		help = t!("cli-nala-installed")
	)]
	pub nala_installed: bool,

	#[clap(short, long, action, help = t!("cli-upgradable"))]
	pub upgradable: bool,

	#[clap(short = 'V', long, action, help = t!("cli-virtual"))]
	pub r#virtual: bool,

	#[clap(short = 'm', long, action, help = t!("cli-machine"))]
	pub machine: bool,
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-search"))]
pub struct Search {
	#[clap(long, action, help = t!("cli-names-only"))]
	pub names_only: bool,

	// Flatten list commands args into search
	#[clap(flatten)]
	pub list_args: List,
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-show"))]
pub struct Show {
	#[clap(
		required = false,
		add = ArgValueCompleter::new(package_completion),
		help = t!("cli-pkg-show")
	)]
	pub pkg_names: Vec<String>,

	#[clap(flatten)]
	pub info: InfoFlags,

	#[clap(short = 'm', long, action, help = t!("cli-machine"))]
	pub machine: bool,
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-policy"))]
pub struct Policy {
	#[clap(
		required = false,
		add = ArgValueCompleter::new(package_completion),
		help = t!("cli-pkg-policy")
	)]
	pub pkg_names: Vec<String>,

	#[clap(short = 'm', long, action, help = t!("cli-machine"))]
	pub machine: bool,
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-clean"))]
pub struct Clean {
	#[clap(long, action, help = t!("cli-clean-lists"))]
	pub lists: bool,

	#[clap(long, action, help = t!("cli-clean-fetch"))]
	pub fetch: bool,
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-download"))]
pub struct Download {
	#[clap(
		add = ArgValueCompleter::new(package_completion),
		help = t!("cli-pkg-download")
	)]
	pub pkg_names: Vec<String>,

	#[clap(long, action, help = t!("cli-clean-fetch"))]
	pub fetch: bool,
}

#[derive(Args, Debug)]
#[clap(
	about = t!("cli-history"),
	args_conflicts_with_subcommands = true
)]
pub struct History {
	#[clap(
		value_name = "ID|last",
		add = ArgValueCompleter::new(history_id_completion),
		help = t!("cli-history-id")
	)]
	pub history_id: Option<HistorySelector>,

	#[clap(subcommand, help = t!("cli-history-action"))]
	pub command: Option<HistoryCommand>,
}

/// Additional actions supported by the history command.
#[derive(Subcommand, Debug)]
#[clap(rename_all = "lower")]
pub enum HistoryCommand {
	#[clap(about = t!("cli-history-undo"))]
	Undo(HistoryTransaction),
	#[clap(about = t!("cli-history-redo"))]
	Redo(HistoryTransaction),
	#[clap(about = t!("cli-history-clear"))]
	Clear(HistoryClear),
}

#[derive(Args, Debug)]
pub struct HistoryTransaction {
	#[clap(
		value_name = "ID|last",
		add = ArgValueCompleter::new(history_id_completion),
		help = t!("cli-history-transaction-id")
	)]
	pub history_id: HistorySelector,
}

#[derive(Args, Debug)]
#[clap(group(
	ArgGroup::new("target")
		.required(true)
		.args(["history_id", "all"])
), about = t!("cli-history-clear"))]
pub struct HistoryClear {
	#[clap(
		value_name = "ID|last",
		conflicts_with = "all",
		add = ArgValueCompleter::new(history_id_completion),
		help = t!("cli-history-clear-id")
	)]
	pub history_id: Option<HistorySelector>,

	#[clap(long, action, help = t!("cli-history-clear-all"))]
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

		value
			.parse::<u32>()
			.map(Self::Id)
			.map_err(|_| t!("history-selector", "value" => value))
	}
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-fetch"))]
pub struct Fetch {
	#[clap(long, action, help = t!("cli-fetch-non-free"))]
	pub non_free: bool,

	#[clap(long, action, help = t!("cli-fetch-https"))]
	pub https_only: bool,

	#[clap(long, action, help = t!("cli-fetch-sources"))]
	pub sources: bool,

	#[clap(
		long,
		num_args = 0..=1,
		default_missing_value = "3",
		help = t!("cli-fetch-auto")
	)]
	pub auto: Option<u8>,

	#[clap(
		short = 'c',
		long,
		action,
		help = t!("cli-fetch-country")
	)]
	pub country: Vec<String>,

	#[clap(long, action, help = t!("cli-fetch-debian"))]
	pub debian: Option<String>,

	#[clap(long, action, help = t!("cli-fetch-ubuntu"))]
	pub ubuntu: Option<String>,

	#[clap(long, action, help = t!("cli-fetch-devuan"))]
	pub devuan: Option<String>,
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-update"))]
pub struct Update {}

#[derive(Args, Debug)]
#[clap(
	about = t!("cli-upgrade"),
	visible_aliases = ["full-upgrade", "safe-upgrade"]
)]
pub struct Upgrade {
	#[clap(long, action, help = t!("cli-print-uris"))]
	pub print_uris: bool,

	#[clap(
		long,
		value_name = "PKG",
		action,
		add = ArgValueCompleter::new(package_completion),
		help = t!("cli-exclude")
	)]
	pub exclude: Vec<String>,

	#[clap(long, action, help = t!("cli-full"))]
	pub full: bool,

	#[clap(long, action, help = t!("cli-no-full"))]
	pub no_full: bool,

	#[clap(long, action, help = t!("cli-safe"))]
	pub safe: bool,

	#[clap(flatten)]
	pub transaction: TransactionFlags,

	#[clap(flatten)]
	pub recommends: InstallFlags,

	#[clap(flatten)]
	pub auto_remove: AutoRemoveFlags,
}

#[derive(Args, Debug)]
#[clap(about = t!("cli-install"))]
pub struct Install {
	#[clap(
		required = false,
		add = ArgValueCompleter::new(package_completion),
		help = t!("cli-pkg-install")
	)]
	pub pkg_names: Vec<String>,

	#[clap(long, action, help = t!("cli-reinstall"))]
	pub reinstall: bool,

	#[clap(
		short = 't',
		long,
		value_name = "RELEASE",
		help = t!("cli-target-release")
	)]
	pub target_release: Option<String>,

	#[clap(flatten)]
	pub transaction: TransactionFlags,

	#[clap(flatten)]
	pub fix_broken: FixBrokenFlags,

	#[clap(flatten)]
	pub recommends: InstallFlags,

	#[clap(flatten)]
	pub auto_remove: AutoRemoveFlags,
}

#[derive(Args, Debug)]
#[clap(
	about = t!("cli-remove"),
	visible_alias = "purge",
	long_about = None
)]
pub struct Remove {
	#[clap(
		add = ArgValueCompleter::new(installed_package_completion),
		help = t!("cli-pkg-remove")
	)]
	pub pkg_names: Vec<String>,

	#[clap(flatten)]
	pub transaction: TransactionFlags,

	#[clap(flatten)]
	pub fix_broken: FixBrokenFlags,

	#[clap(flatten)]
	pub auto_remove: AutoRemoveFlags,
}

#[derive(Args, Debug)]
#[clap(
	about = t!("cli-autoremove"),
	visible_alias = "autopurge",
	long_about = None
)]
pub struct AutoRemove {
	#[clap(long, action, help = t!("cli-remove-config"))]
	pub remove_config: bool,

	#[clap(flatten)]
	pub transaction: TransactionFlags,
}

#[derive(Args, Debug)]
pub struct Moo {
	#[clap(long, hide = true, action)]
	pub help: bool,

	#[clap(long, hide = true, action)]
	pub update: bool,

	#[clap(long, hide = true, action)]
	pub no_update: bool,
}
