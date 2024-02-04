use std::env;
use std::io::Error;

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::Bash;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
	let Some(outdir) = env::var_os("OUT_DIR") else {
		return Ok(());
	};

	// let mut cmd = NalaParser::command();
	let path = generate_to(
		Bash,
		&mut NalaParser::command(), // We need to specify what generator to use
		"nala-rs",                  // We need to specify the bin name manually
		outdir,                     // We need to specify where to write to
	)?;

	println!("cargo:warning=completion file is generated: {path:?}");

	Ok(())
}
