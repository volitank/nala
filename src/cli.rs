use clap::{Arg, ArgAction, Command};

pub fn build() -> Command<'static> {
	Command::new("nala-rs")
		.about("Commandline front-end for libapt-pkg")
		.version("0.1.0")
		.author("Blake Lee <blake@volian.org>")
		.subcommand_required(false)
		.arg_required_else_help(true)
		.disable_version_flag(true)
		.arg(
			Arg::new("license")
				.long("license")
				.help("Print license information")
				.action(ArgAction::SetTrue)
				.display_order(1),
		)
		.arg(
			Arg::new("debug")
				.long("debug")
				.help("Print debug statements for solving issues")
				.action(ArgAction::SetTrue)
				.global(true)
				.display_order(1),
		)
		.arg(
			Arg::new("verbose")
				.long("verbose")
				.help("Disable scrolling text and print extra information")
				.action(ArgAction::SetTrue)
				.takes_value(true)
				.global(true)
				.display_order(1),
		)
		.arg(
			Arg::new("config")
				.long("config")
				.help("Specify a different configuration file")
				.action(ArgAction::Set)
				.takes_value(true)
				.global(true)
				.display_order(1),
		)
		.subcommand(
			Command::new("list")
				.about("List packages based on package names.")
				.arg(
					Arg::new("pkg_names")
						.help("Package names to list")
						.action(ArgAction::Append)
						.multiple_occurrences(true)
						.takes_value(true)
						.required(false),
				)
				.arg(
					Arg::new("description")
						.long("description")
						.help("Print the full description of each package")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("summary")
						.long("summary")
						.help("Print the summary of each package")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("all_versions")
						.short('a')
						.long("all-versions")
						.help("Show all versions of a package")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("installed")
						.short('i')
						.long("installed")
						.help("List only packages that are installed")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("nala_installed")
						.short('N')
						.long("nala-installed")
						.help("List only packages explicitly installed with Nala")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("upgradable")
						.short('u')
						.long("upgradable")
						.help("List only packages that are upgradable")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("virtual")
						.short('V')
						.long("virtual")
						.help("List only virtual packages")
						.action(ArgAction::SetTrue),
				),
		)
		.subcommand(
			Command::new("search")
				.about("Search package names and descriptions.")
				.arg(
					Arg::new("pkg_names")
						.help("Package names to list")
						.action(ArgAction::Append)
						.multiple_occurrences(true)
						.takes_value(true)
						.required(false),
				)
				.arg(
					Arg::new("names")
						.long("names")
						.help("Search only Package names")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("description")
						.long("description")
						.help("Print the full description of each package")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("summary")
						.long("summary")
						.help("Print the summary of each package")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("all_versions")
						.short('a')
						.long("all-versions")
						.help("Show all versions of a package")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("installed")
						.short('i')
						.long("installed")
						.help("List only packages that are installed")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("nala_installed")
						.short('N')
						.long("nala-installed")
						.help("List only packages explicitly installed with Nala")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("upgradable")
						.short('u')
						.long("upgradable")
						.help("List only packages that are upgradable")
						.action(ArgAction::SetTrue),
				)
				.arg(
					Arg::new("virtual")
						.short('V')
						.long("virtual")
						.help("List only virtual packages")
						.action(ArgAction::SetTrue),
				),
		)
}
