use anyhow::Result;
use rust_apt::new_cache;

use crate::config::Config;
use crate::libnala::HistoryFile;
use crate::summary::display_summary;

pub async fn history(config: &Config, cmd: &crate::cli::parser::History) -> Result<()> {
	let history_file = HistoryFile::from_config(config).await?;

	if let Some(hist) = &cmd.hist_command {
		match hist {
			crate::cli::parser::HistoryCmd::Info { id } => {
				let cache = new_cache!()?;
				display_summary(&cache, config, history_file.get(id)?).await?;
			},
			crate::cli::parser::HistoryCmd::Redo { id: _ } => todo!(),
			crate::cli::parser::HistoryCmd::Undo { id } => {
				// let cache = new_cache!()?;
				let entry = history_file.get(id)?;
				let mut table = history_file.table();

				table.add_row(history_file.get(id)?.as_row());
				println!("{table}");

				println!("Packages:");
				for pkg in entry.pkgs() {
					println!("{pkg:?}")
				}
			},
		}
	} else {
		let mut table = history_file.table();
		for entry in history_file.iter() {
			table.add_row(entry.as_row());
		}

		println!("{table}");
	}
	Ok(())
}

pub static NALA: &str = r"
//       /\     /\
//      {  `---'  }
//      {  O   O  }
//      ~~>  V  <~~
//        `-----'____
//        /     \    \_
//       {       }\  )_\_   _
//       |  \_/  |/ /  \_\_/ )
//        \__/  /(_/     \__/
//          (__/
";
