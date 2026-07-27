use std::path::PathBuf;

use clap::{ColorChoice, Parser};

use super::commands::Commands;
use crate::t;

#[derive(Parser, Debug)]
#[clap(name = "nala")]
#[clap(author = "Blake Lee <blake@volian.org>")]
#[clap(version)]
#[clap(about = t!("cli-about"), long_about = None)]
pub struct NalaParser {
	#[clap(global = true, short, long, action, help = t!("cli-license"))]
	pub license: bool,

	#[clap(global = true, short, long, action, help = t!("cli-verbose"))]
	pub verbose: bool,
	#[clap(global = true, short, long, action, help = t!("cli-debug"))]
	pub debug: bool,

	#[clap(
		short,
		long,
		value_parser,
		value_name = "FILE",
		help = t!("cli-config")
	)]
	pub config: Option<PathBuf>,

	/// Override the history directory for tests and isolated fixtures
	#[clap(global = true, long, hide = true, value_name = "DIR")]
	pub history_dir: Option<PathBuf>,

	#[clap(
		global = true,
		long,
		action,
		conflicts_with = "no_tui",
		help = t!("cli-tui")
	)]
	pub tui: bool,

	#[clap(
		global = true,
		long,
		action,
		conflicts_with = "tui",
		help = t!("cli-no-tui")
	)]
	pub no_tui: bool,

	#[clap(
		global = true,
		short = 'o',
		long,
		action,
		help = t!("cli-option")
	)]
	pub option: Vec<String>,

	#[clap(
		global = true,
		long,
		default_value = "auto",
		help = t!("cli-color")
	)]
	pub color: ColorChoice,

	#[clap(subcommand)]
	pub command: Option<Commands>,
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
		let parsed = NalaParser::try_parse_from(["nala", "install", "-y", "demo"]).unwrap();
		let Some(Commands::Install(args)) = parsed.command else {
			panic!("expected install command");
		};
		assert!(args.transaction.assume_yes);
		assert!(!args.transaction.assume_no);

		let parsed = NalaParser::try_parse_from(["nala", "install", "-n", "demo"]).unwrap();
		let Some(Commands::Install(args)) = parsed.command else {
			panic!("expected install command");
		};
		assert!(args.transaction.assume_no);
		assert!(!args.transaction.assume_yes);

		assert!(
			NalaParser::try_parse_from(["nala", "install", "--assume-yes", "--assume-no", "demo",])
				.is_err()
		);
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

		let Some(Commands::Install(args)) = parsed.command else {
			panic!("expected install command");
		};
		assert!(args.recommends.install_recommends);
		assert!(args.recommends.no_install_suggests);
		assert_eq!(args.target_release.as_deref(), Some("testing"));
		assert!(args.fix_broken.no_fix_broken);

		assert!(
			NalaParser::try_parse_from([
				"nala",
				"install",
				"--install-suggests",
				"--no-install-suggests",
				"demo",
			])
			.is_err()
		);
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

		let Some(Commands::Remove(args)) = parsed.command else {
			panic!("expected remove command");
		};
		assert!(args.transaction.remove_essential);
		assert!(args.auto_remove.no_auto_remove);

		let parsed =
			NalaParser::try_parse_from(["nala", "install", "--autoremove", "demo"]).unwrap();
		let Some(Commands::Install(args)) = parsed.command else {
			panic!("expected install command");
		};
		assert!(args.auto_remove.auto_remove);

		assert!(
			NalaParser::try_parse_from([
				"nala",
				"install",
				"--auto-remove",
				"--no-autoremove",
				"demo",
			])
			.is_err()
		);
	}

	#[test]
	fn update_flags_parse() {
		let parsed = NalaParser::try_parse_from(["nala", "install", "--update", "demo"]).unwrap();
		let Some(Commands::Install(args)) = parsed.command else {
			panic!("expected install command");
		};
		assert!(args.transaction.update);
		assert!(!args.transaction.no_update);

		let parsed = NalaParser::try_parse_from(["nala", "upgrade", "--no-update"]).unwrap();
		let Some(Commands::Upgrade(args)) = parsed.command else {
			panic!("expected upgrade command");
		};
		assert!(args.transaction.no_update);
		assert!(!args.transaction.update);

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
	fn all_arches_flags_parse() {
		let parsed = NalaParser::try_parse_from(["nala", "list", "--all-arches"]).unwrap();
		let Some(Commands::List(args)) = parsed.command else {
			panic!("expected list command");
		};
		assert!(args.info.all_arches);

		let parsed = NalaParser::try_parse_from(["nala", "search", "-A", "demo"]).unwrap();
		let Some(Commands::Search(args)) = parsed.command else {
			panic!("expected search command");
		};
		assert!(args.list_args.info.all_arches);
	}

	#[test]
	fn virtual_flags_parse() {
		let parsed = NalaParser::try_parse_from(["nala", "list", "--virtual"]).unwrap();
		let Some(Commands::List(args)) = parsed.command else {
			panic!("expected list command");
		};
		assert!(args.r#virtual);

		let parsed = NalaParser::try_parse_from(["nala", "search", "-V", "demo"]).unwrap();
		let Some(Commands::Search(args)) = parsed.command else {
			panic!("expected search command");
		};
		assert!(args.list_args.r#virtual);
	}

	#[test]
	fn simple_summary_flags_parse() {
		let parsed = NalaParser::try_parse_from(["nala", "install", "--simple", "demo"]).unwrap();
		let Some(Commands::Install(args)) = parsed.command else {
			panic!("expected install command");
		};
		assert!(args.transaction.simple);
	}
}
