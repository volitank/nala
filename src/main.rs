use std::path::Path;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use config::logger::LogOptions;
use config::Level;
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
		if let Some(apt_errors) = err.downcast_ref::<AptErrors>() {
			for error in apt_errors.iter() {
				if error.is_error {
					error!("{}", error.msg.replace("E: ", ""));
				} else {
					warn!("{}", error.msg.replace("W: ", ""));
				};
			}
		} else {
			error!("{err:?}");
		}
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
}

fn get_config() -> Result<(ArgMatches, NalaParser, Config)> {
	let args = NalaParser::command().get_matches();
	let derived = NalaParser::from_arg_matches(&args)?;

	let config_file = match derived.config {
		Some(ref conf_file) => conf_file,
		None => Path::new("/etc/nala/nala.conf"),
	};

	let config = match Config::new(config_file) {
		Ok(config) => config,
		Err(err) => {
			eprintln!("Warning: {err}");
			Config::default()
		},
	};

	Ok((args, derived, config))
}

#[tokio::main]
async fn main_nala(args: ArgMatches, derived: NalaParser, config: &mut Config) -> Result<()> {
	if derived.license {
		println!("Not Yet Implemented.");
		return Ok(());
	}

	let options = LogOptions::new(Level::Info, Box::new(std::io::stderr()));
	let logger = crate::config::setup_logger(options);

	if let (Some((name, cmd)), Some(command)) = (args.subcommand(), derived.command) {
		config.command = name.to_string();
		config.load_args(cmd)?;

		for (config, level) in [
			(config.verbose(), crate::config::Level::Verbose),
			(config.debug(), crate::config::Level::Debug),
		] {
			if config {
				logger.lock().unwrap().set_level(level);
			}
		}
		command.run(config).await?;
	} else {
		NalaParser::command().print_help()?;
		bail!("Subcommand not found")
	}
	Ok(())
}
