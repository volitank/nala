use clap::Args;

use crate::t;

/// Flags common to all transactional subcommands (install, remove, upgrade,
/// autoremove).
#[derive(Args, Default, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct TransactionFlags {
	#[clap(long, action, help = t!("cli-download-only"))]
	pub download_only: bool,

	#[clap(long, action, help = t!("cli-simple"))]
	pub simple: bool,

	#[clap(
		long,
		action,
		conflicts_with = "no_update",
		help = t!("cli-update-first")
	)]
	pub update: bool,

	#[clap(
		long,
		action,
		conflicts_with = "update",
		help = t!("cli-no-update")
	)]
	pub no_update: bool,

	#[clap(long, action, help = t!("cli-allow-unauthenticated"))]
	pub allow_unauthenticated: bool,

	#[clap(
		short = 'y',
		long,
		action,
		conflicts_with = "assume_no",
		help = t!("cli-assume-yes")
	)]
	pub assume_yes: bool,

	#[clap(
		short = 'n',
		long,
		action,
		conflicts_with = "assume_yes",
		help = t!("cli-assume-no")
	)]
	pub assume_no: bool,

	#[clap(long, action, help = t!("cli-remove-essential"))]
	pub remove_essential: bool,

	#[clap(long, action, help = t!("cli-purge"))]
	pub purge: bool,
}

/// Fix broken flags (install and remove only).
#[derive(Args, Default, Debug)]
pub struct FixBrokenFlags {
	#[clap(
		short = 'f',
		long,
		action,
		conflicts_with = "no_fix_broken",
		help = t!("cli-fix-broken")
	)]
	pub fix_broken: bool,

	#[clap(
		long,
		action,
		conflicts_with = "fix_broken",
		help = t!("cli-no-fix-broken")
	)]
	pub no_fix_broken: bool,
}

/// Recommends/suggests flags (install and upgrade only).
#[derive(Args, Default, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct InstallFlags {
	#[clap(
		long,
		action,
		conflicts_with = "no_install_recommends",
		help = t!("cli-install-recommends")
	)]
	pub install_recommends: bool,

	#[clap(
		long,
		action,
		conflicts_with = "install_recommends",
		help = t!("cli-no-install-recommends")
	)]
	pub no_install_recommends: bool,

	#[clap(
		long,
		action,
		conflicts_with = "no_install_suggests",
		help = t!("cli-install-suggests")
	)]
	pub install_suggests: bool,

	#[clap(
		long,
		action,
		conflicts_with = "install_suggests",
		help = t!("cli-no-install-suggests")
	)]
	pub no_install_suggests: bool,
}

#[derive(Args, Debug)]
pub struct InfoFlags {
	#[clap(short = 'a', long, action, help = t!("cli-all-versions"))]
	pub all_versions: bool,

	#[clap(short = 'A', long, action, help = t!("cli-all-arches"))]
	pub all_arches: bool,
}

#[derive(Args, Default, Debug)]
pub struct AutoRemoveFlags {
	#[clap(
		long,
		visible_alias = "autoremove",
		action,
		conflicts_with = "no_auto_remove",
		help = t!("cli-auto-remove")
	)]
	pub auto_remove: bool,

	#[clap(
		long,
		visible_alias = "no-autoremove",
		action,
		conflicts_with = "auto_remove",
		help = t!("cli-no-auto-remove")
	)]
	pub no_auto_remove: bool,
}
