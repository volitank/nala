use clap::Args;

/// Flags common to all transactional subcommands (install, remove, upgrade,
/// autoremove).
#[derive(Args, Default, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct TransactionFlags {
	/// Only download packages.
	#[clap(long, action)]
	pub download_only: bool,

	/// Display a simpler and more condensed transaction summary.
	#[clap(long, action)]
	pub simple: bool,

	/// Update package lists before running the command.
	#[clap(long, action, conflicts_with = "no_update")]
	pub update: bool,

	/// Do NOT update package lists before running the command.
	#[clap(long, action, conflicts_with = "update")]
	pub no_update: bool,

	/// Allow Nala to install packages that can't be hashsum verified
	#[clap(long, action)]
	pub allow_unauthenticated: bool,

	/// Assume yes for all prompts.
	#[clap(short = 'y', long, action, conflicts_with = "assume_no")]
	pub assume_yes: bool,

	/// Assume no for all prompts.
	#[clap(short = 'n', long, action, conflicts_with = "assume_yes")]
	pub assume_no: bool,

	/// Allow the removal of essential packages.
	#[clap(long, action)]
	pub remove_essential: bool,

	/// Remove config files for any package set to be removed.
	#[clap(long, action)]
	pub purge: bool,
}

/// Fix broken flags (install and remove only).
#[derive(Args, Default, Debug)]
pub struct FixBrokenFlags {
	/// Try to fix broken packages.
	#[clap(short = 'f', long, action, conflicts_with = "no_fix_broken")]
	pub fix_broken: bool,

	/// Do NOT try to fix broken packages.
	#[clap(long, action, conflicts_with = "fix_broken")]
	pub no_fix_broken: bool,
}

/// Recommends/suggests flags (install and upgrade only).
#[derive(Args, Default, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct InstallFlags {
	/// Install recommended packages.
	#[clap(long, action, conflicts_with = "no_install_recommends")]
	pub install_recommends: bool,

	/// Do NOT install recommended packages.
	#[clap(long, action, conflicts_with = "install_recommends")]
	pub no_install_recommends: bool,

	/// Install suggested packages.
	#[clap(long, action, conflicts_with = "no_install_suggests")]
	pub install_suggests: bool,

	/// Do NOT install suggested packages.
	#[clap(long, action, conflicts_with = "install_suggests")]
	pub no_install_suggests: bool,
}

#[derive(Args, Debug)]
pub struct InfoFlags {
	/// Show all versions of a package
	#[clap(short = 'a', long, action)]
	pub all_versions: bool,

	/// Show packages for all configured architectures
	#[clap(short = 'A', long, action)]
	pub all_arches: bool,
}

#[derive(Args, Default, Debug)]
pub struct AutoRemoveFlags {
	/// Additionally remove unnecessary packages.
	#[clap(
		long,
		visible_alias = "autoremove",
		action,
		conflicts_with = "no_auto_remove"
	)]
	pub auto_remove: bool,

	/// Do NOT remove unnecessary packages.
	#[clap(
		long,
		visible_alias = "no-autoremove",
		action,
		conflicts_with = "auto_remove"
	)]
	pub no_auto_remove: bool,
}
