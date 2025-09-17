use std::path::Path;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use config::color::{self};
use config::logger;
use rust_apt::error::AptErrors;

mod cli;
mod config;
mod deb;
mod download;
mod dpkg;
mod fs;
mod glob;
mod hashsum;
mod libnala;
mod summary;
mod table;
mod tui;

use crate::cli::NalaParser;
use crate::config::Config;

// This is basically the error handling prior to
// configuring the logger and such.
macro_rules! rip {
	($result:expr) => {
		match $result {
			Ok(ok) => ok,
			Err(err) => {
				eprintln!("\x1b[1;91mError:\x1b[0m {err:?}");
				return ExitCode::FAILURE;
			},
		}
	};
}

fn main() -> ExitCode {
	// dbg!(color::get_color().as_ref());
	rip!(logger::Logger::default().init());
	let (args, derived, mut config) = rip!(get_config());

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
		if let Some(apt_errors) = err.downcast_ref::<AptErrors>() {
			for error in apt_errors.iter() {
				if error.is_error {
					crate::error!("{}", error.msg.replace("E: ", ""));
				} else {
					crate::warning!("{}", error.msg.replace("W: ", ""));
				};
			}
		} else {
			crate::error!("{err:?}");
		}
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
}

fn get_config() -> Result<(ArgMatches, NalaParser, Config)> {
	let args = NalaParser::command().get_matches();

	let derived = NalaParser::from_arg_matches(&args)?;

	let config = Config::read_config(match &derived.config {
		Some(conf_file) => conf_file,
		None => Path::new("/etc/nala/nala.conf"),
	});

	Ok((args, derived, config))
}

#[tokio::main]
async fn main_nala(args: ArgMatches, derived: NalaParser, config: &mut Config) -> Result<()> {
	if derived.license {
		println!("Not Yet Implemented.");
		return Ok(());
	}

	if let (Some((name, cmd)), Some(command)) = (args.subcommand(), derived.command) {
		config.command = name.to_string();
		config.load_args(cmd)?;

		// for (config, level) in [
		// 	(config.verbose(), log::Level::Trace),
		// 	(config.debug(), log::Level::Debug),
		// ] {
		// 	if config {
		// 		let logger = Logger::default().init();
		// 	}
		// }
		command.run(config).await?;
	} else {
		NalaParser::command().print_help()?;
		bail!("Subcommand not found")
	}
	Ok(())
}
