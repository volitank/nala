use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{CommandFactory, FromArgMatches};
use downloader::download;
use history::history_test;

mod clean;
mod cli;
mod colors;
mod config;
mod downloader;
mod history;
mod list;
mod show;
mod util;
use crate::clean::clean;
use crate::cli::NalaParser;
use crate::colors::Color;
use crate::config::Config;
use crate::list::{list, search};
use crate::show::show;

fn main() -> ExitCode {
	// Setup default color to print pretty even if the config fails
	let color = Color::default();
	if let Err(err) = main_nala(&color) {
		color.error(&format!("{err:?}"));
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
}

fn main_nala(color: &Color) -> Result<()> {
	let args = NalaParser::command().get_matches();
	let derived = NalaParser::from_arg_matches(&args)?;

	let conf_file = match derived.config {
		Some(path) => path,
		None => PathBuf::from("/etc/nala/nala.conf"),
	};

	let mut config = match Config::new(&conf_file) {
		Ok(config) => config,
		Err(err) => {
			// Warn the user of the error and assume defaults
			// TODO: Decide how and when to error instead of warn
			// At the moment this will always warn and then default
			color.warn(&format!("{err:?}"));
			Config::default()
		},
	};

	if derived.license {
		println!("Not Yet Implemented.");
		return Ok(());
	}

	if let Some((name, cmd)) = args.subcommand() {
		config.load_args(cmd);
		match name {
			"list" => list(&config)?,
			"search" => search(&config)?,
			"show" => show(&config)?,
			"clean" => clean(&config)?,
			"download" => download(&config)?,
			"history" => history_test(&config)?,
			// Match other subcommands here...
			_ => bail!("Unknown error in the argument parser"),
		}
	} else {
		NalaParser::command().print_help()?;
		bail!("Subcommand not found")
	}
	Ok(())
}
