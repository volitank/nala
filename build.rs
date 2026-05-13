use clap::CommandFactory;
use clap_mangen as man;

mod flags {
	include!("src/cli/flags.rs");
}
mod completion {
	use std::ffi::OsStr;

	use clap_complete::engine::CompletionCandidate;

	pub fn package_completion(_: &OsStr) -> Vec<CompletionCandidate> { Vec::new() }

	pub fn installed_package_completion(_: &OsStr) -> Vec<CompletionCandidate> { Vec::new() }

	pub fn history_id_completion(_: &OsStr) -> Vec<CompletionCandidate> { Vec::new() }
}
mod commands {
	include!("src/cli/commands.rs");
}
mod parser {
	include!("src/cli/parser.rs");
}

use parser::NalaParser;

macro_rules! gen {
	($label:literal, $out_dir:expr, $code:block) => {{
		let path = $out_dir.join($label);
		let result = { $code };
		println!("cargo:warning={} files are generated: {path:?}", $label,);
		result
	}};
}

fn main() -> Result<(), std::io::Error> {
	let out_dir =
		std::path::PathBuf::from(std::env::var_os("OUT_DIR").ok_or(std::io::ErrorKind::NotFound)?);

	let parser = NalaParser::command();

	gen!("Manpage", out_dir, {
		man::generate_to(parser, &out_dir)?;
	});

	gen!("Markdown", out_dir, {
		let opts = clap_markdown::MarkdownOptions::new()
			.show_footer(false)
			.show_table_of_contents(false)
			.title("Nala".to_string());
		let markdown = clap_markdown::help_markdown_custom::<NalaParser>(&opts);
		std::fs::write(out_dir.join("nala.md"), markdown)?;
	});

	Ok(())
}
