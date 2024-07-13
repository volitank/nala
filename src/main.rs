use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use history::history_test;
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
mod tui;
mod util;

use crate::clean::clean;
use crate::cli::NalaParser;
use crate::colors::Color;
use crate::config::Config;
use crate::downloader::download;
use crate::fetch::fetch;
use crate::list::{list, search};
use crate::show::show;
use crate::update::update;

fn main() -> ExitCode {
	// Setup default color to print pretty even if the config fails
	let color = Color::default();

	let (args, derived, mut config) = match get_config() {
		Ok(conf) => conf,
		Err(err) => {
			color.error(&format!("{err:?}"));
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
			config.color.error(&format!("{err:?}"));
			return ExitCode::FAILURE;
		};

		for error in apt_errors.iter() {
			if error.is_error {
				config.color.error(&error.msg.replace("E: ", ""));
			} else {
				config.color.warn(&error.msg.replace("W: ", ""));
			}
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

	if let Some((name, cmd)) = args.subcommand() {
		config.command = name.to_string();
		config.load_args(cmd, derived.command);
		match name {
			"list" => list(config)?,
			"search" => search(config)?,
			"show" => show(config)?,
			"clean" => clean(config)?,
			"download" => download(config)?,
			"history" => history_test(config)?,
			"fetch" => fetch(config)?,
			"update" => update(config)?,
			// Match other subcommands here...
			_ => bail!("Unknown error in the argument parser"),
		}
	} else {
		NalaParser::command().print_help()?;
		bail!("Subcommand not found")
	}
	Ok(())
}
