use std::path::Path;
use std::process::ExitCode;

use anyhow::{bail, Result};
use clap::{ArgMatches, CommandFactory, FromArgMatches};
use cli::Commands;
use cmd::Operation;
use config::logger::LogOptions;
use config::{Level, Paths};
use deb::DebFile;
use rust_apt::cache::Upgrade;
use rust_apt::error::AptErrors;
use rust_apt::{new_cache, PackageSort};
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
mod summary;
mod table;
mod tui;
mod util;

use crate::cli::NalaParser;
use crate::cmd::{clean, fetch, history, list_packages, mark_cli_pkgs, show, update, upgrade};
use crate::config::Config;
use crate::download::download;

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

pub async fn system(config: &Config) -> Result<()> {
	// This downloads all of the pkgs into the archives directory
	// let cache = rust_apt::new_cache!()?;
	// println!("Cache Total Pkgs: {}", cache.iter().count());

	// let mut downloader = Downloader::new(config)?;

	// let versions = cache
	// 	.iter()
	// 	.filter_map(|p| {
	// 		let v = p.installed()?;
	// 		if v.is_downloadable() {
	// 			Some(v)
	// 		} else {
	// 			None
	// 		}
	// 	})
	// 	.collect::<Vec<_>>();

	// for ver in &versions {
	// 	downloader.add_version(ver, config).await?;
	// }

	// downloader.run(config, false).await?;

	let archive = config.get_path(&Paths::Archive);
	let mut debs = vec![];
	for entry in std::fs::read_dir(archive)? {
		let entry = entry?;
		let metadata = entry.metadata()?;

		let path = entry.path();

		// If it's a directory, recurse into it
		if metadata.is_dir() {
			continue;
		}

		debs.push(path.to_string_lossy().to_string())
	}

	let cache = new_cache!(&debs)?;
	let filtered_pkgs = cache
		.packages(&PackageSort::default().installed())
		.filter_map(|pkg| {
			let version = pkg.installed()?;
			let file = version
				.version_files()
				.filter_map(|vf| {
					let pf = vf.package_file();
					// Instead of archive we could match the filename
					// pf.filename().unwrap().contains(Paths::Archive.default_path())
					if pf.archive()? == "local-deb" {
						Some(pf.filename()?.to_string())
					} else {
						None
					}
				})
				.next()?;
			Some((version, file))
		})
		.collect::<Vec<_>>();

	let mut pb = tui::NalaProgressBar::new(config, true)?;
	let mut set = tokio::task::JoinSet::new();
	for (_, file) in filtered_pkgs {
		set.spawn(DebFile::new(file));
	}

	let files = pb.join(set).await?;
	for file in files {
		file.store().await?;
	}
	Ok(())
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

		match command {
			Commands::List(_) | Commands::Search(_) => {
				let cache = new_cache!()?;
				list_packages(
					config,
					if config.command == "search" {
						glob::regex_pkgs(config, &cache)?.only_pkgs()
					} else if config.pkg_names().is_ok() {
						glob::pkgs_with_modifiers(config.pkg_names()?, config, &cache)?.only_pkgs()
					} else {
						cache.packages(&glob::get_sorter(config)).collect()
					},
				)?;
			},
			Commands::Show(_) => show(config)?,
			Commands::Clean(_) => clean(config)?,
			Commands::Download(_) => download(config).await?,
			Commands::History(_) => history(config).await?,
			Commands::Fetch(_) => fetch(config)?,
			Commands::Update(_) => update(config).await?,
			Commands::Upgrade(_) => {
				upgrade(
					config,
					// SafeUpgrade takes precedence.
					if config.get_bool("safe", false) {
						Upgrade::SafeUpgrade
					} else if config.get_no_bool("full", false) {
						Upgrade::FullUpgrade
					} else {
						Upgrade::Upgrade
					},
				)
				.await?
			},
			Commands::Install(_) => mark_cli_pkgs(config, Operation::Install).await?,
			Commands::Remove(_) => mark_cli_pkgs(config, Operation::Remove).await?,
			Commands::AutoRemove(_) => {
				sudo_check(config)?;
				crate::summary::commit(new_cache!()?, config).await?;
			},
			Commands::System(_) => system(config).await?,
		}
	} else {
		NalaParser::command().print_help()?;
		bail!("Subcommand not found")
	}
	Ok(())
}
