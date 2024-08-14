use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use cli::Commands;
use colors::Theme;
use history::history;
use rust_apt::error::AptErrors;

mod cli;
mod fetch;
mod history;
mod list;
mod show;
mod update;

mod clean;
mod colors;
mod config;
mod downloader;
mod dpkg;
mod install;
mod table;
mod tui;
mod upgrade;
mod util;

use crate::clean::clean;
use crate::cli::NalaParser;
use crate::config::Config;
use crate::downloader::download;
use crate::fetch::fetch;
use crate::install::install;
use crate::list::{list, search};
use crate::show::show;
use crate::update::update;
use crate::upgrade::upgrade;

fn main() -> ExitCode {
	let (args, derived, mut config) = match get_config() {
		Ok(conf) => conf,
		Err(err) => {
			eprintln!("\x1b[1;91mError:\x1b[0m {err:?}");
			return ExitCode::FAILURE;
		},
	};

	// TODO: We should probably have a notification system
	// to pipe messages that aren't critical back to here
	// to display before the program exists. For example
	// Notice: 'pkg' was not found
	// Notice: There are 2 additional records.
	// This can simplify some parts of the code like list/search

	// For all other errors use the color defined in the config.
	if let Err(err) = main_nala(args, derived, &mut config) {
		// Guard clause in cause it is not AptErrors
		// In this case just print it nicely
		let Some(apt_errors) = err.downcast_ref::<AptErrors>() else {
			config.stderr(Theme::Error, &format!("{err:?}"));
			return ExitCode::FAILURE;
		};

		for error in apt_errors.iter() {
			let (theme, msg) = if error.is_error {
				(Theme::Error, error.msg.replace("E: ", ""))
			} else {
				(Theme::Warning, error.msg.replace("W: ", ""))
			};
			config.stderr(theme, &msg);
		}
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
}

fn get_config() -> Result<(ArgMatches, NalaParser, Config)> {
	let args = NalaParser::command().get_matches();
	let derived = NalaParser::from_arg_matches(&args)?;

	let config = match derived.config {
		Some(ref conf_file) => Config::new(conf_file)?,
		None => Config::new(&PathBuf::from("/etc/nala/nala.conf"))?,
	};

	Ok((args, derived, config))
}

fn main_nala(args: ArgMatches, derived: NalaParser, config: &mut Config) -> Result<()> {
	if derived.license {
		println!("Not Yet Implemented.");
		return Ok(());
	}

	if let (Some((name, cmd)), Some(command)) = (args.subcommand(), derived.command) {
		config.command = name.to_string();
		config.load_args(cmd);
		match command {
			Commands::List(_) => list(config)?,
			Commands::Search(_) => search(config)?,
			Commands::Show(_) => show(config)?,
			Commands::Clean(_) => clean(config)?,
			Commands::Download(_) => download(config)?,
			Commands::History(_) => history(config)?,
			Commands::Fetch(_) => fetch(config)?,
			Commands::Update(_) => update(config)?,
			Commands::Upgrade(_) => upgrade(config)?,
			Commands::Install(_) => install(config)?,
		}
	} else {
		NalaParser::command().print_help()?;
		bail!("Subcommand not found")
	}
	Ok(())
}
