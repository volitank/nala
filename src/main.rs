use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{ArgMatches, CommandFactory};
use cli::Commands;
use cmd::Operation;
use config::logger::LogOptions;
use config::Level;
use rust_apt::cache::Upgrade;
use rust_apt::error::AptErrors;
use rust_apt::new_cache;
use util::sudo_check;

mod cli;
mod cmd;
mod config;
mod deb;
mod download;
mod dpkg;
mod fs;
mod glob;
mod hashsum;
mod libnala;
mod progress;
mod summary;
mod table;
mod terminal;
mod tui;
mod util;

use crate::cli::NalaParser;
use crate::cmd::{
	clean, fetch, history, list_packages, mark_cli_pkgs, policy, show, update, upgrade,
};
use crate::config::Config;
use crate::download::download;

fn main() -> ExitCode {
	let (args, derived, mut config) = match config::bootstrap() {
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
		} else if format!("{err:?}") != "Subcommand not supplied" {
			error!("{err:?}");
		}
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
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

		{
			let mut logger_guard = logger.lock().unwrap();
			for (enabled, level) in [
				(config.verbose(), crate::config::Level::Verbose),
				(config.debug(), crate::config::Level::Debug),
			] {
				if enabled {
					logger_guard.set_level(level);
				}
			}
		}

		if config.debug() {
			debug!("{config:?}");
		}

		match command {
			Commands::List(_) | Commands::Search(_) => {
				let cache = new_cache!()?;
				let mut missing = Vec::new();
				let packages = if config.command == "search" {
					glob::regex_pkgs(config, &cache)?
				} else {
					match config.pkg_names() {
						Ok(names) => {
							let selection = glob::pkgs_with_modifiers(names, config, &cache)?;
							let (packages, selection_missing) =
								selection.into_packages_and_missing();
							missing = selection_missing;
							packages
						},
						Err(_) => cache.packages(&glob::get_sorter(config)).collect(),
					}
				};

				list_packages(config, packages)?;
				glob::log_missing_notices(&missing);
			},
			Commands::Show(_) => show(config)?,
			Commands::Policy(_) => policy(config)?,
			Commands::Clean(_) => clean(config)?,
			Commands::Download(_) => download(config).await?,
			Commands::History(args) => history(config, &args).await?,
			Commands::Fetch(_) => fetch(config)?,
			Commands::Update(_) => update(config).await?,
			Commands::Upgrade(_) => upgrade(config, upgrade_mode(config)).await?,
			Commands::Install(args) => {
				let operation = if args.reinstall {
					Operation::Reinstall
				} else {
					Operation::Install
				};
				mark_cli_pkgs(config, operation).await?;
			},
			Commands::Remove(_) => mark_cli_pkgs(config, Operation::Remove).await?,
			Commands::AutoRemove(_) => {
				sudo_check(config)?;
				crate::summary::commit(new_cache!()?, config).await?;
			},
		}
	} else {
		NalaParser::command().print_help()?;
		bail!("Subcommand not supplied")
	}
	Ok(())
}

fn upgrade_mode(config: &Config) -> Upgrade {
	// SafeUpgrade takes precedence.
	if config.get_bool("safe", false) {
		Upgrade::SafeUpgrade
	} else if config.get_no_bool("full", false) {
		Upgrade::FullUpgrade
	} else {
		Upgrade::Upgrade
	}
}
