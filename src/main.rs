use std::process::ExitCode;

use anyhow::{anyhow, Result};

mod cli;
mod colors;
mod config;
mod list;
mod util;
use crate::colors::Color;
use crate::config::Config;
use crate::list::list;

fn main() -> ExitCode {
	let mut color = Color::default();
	if let Err(err) = main_nala(&mut color) {
		color.error(&format!("{err:?}"));
		return ExitCode::FAILURE;
	}
	ExitCode::SUCCESS
}

fn main_nala(color: &mut Color) -> Result<()> {
	let args = cli::build().get_matches();
	let conf_file = match args.get_one::<String>("config") {
		Some(string) => string,
		None => "/etc/nala/nala.conf",
	};
	let mut config = Config::new(color, conf_file);
	color.update_from_config(&config)?;

	if *args.get_one::<bool>("license").unwrap_or(&false) {
		println!("Not Yet Implemented.");
		return Ok(());
	}

	match args.subcommand() {
		Some(("list", cmd)) => {
			config.load_args(cmd);
			list(&config, color)?;
		},
		// Match other subcommands here...
		_ => return Err(anyhow!("Unknown error in the argument parser")),
	}
	Ok(())
}
