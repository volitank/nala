mod model;
mod replay;
mod store;
mod view;

pub use model::HistoryEntry;
pub use store::{get_history, next_history_id};

use anyhow::Result;
use rust_apt::new_cache;

use crate::cli::{History, HistoryCommand};
use crate::config::Config;
use crate::terminal::{use_tui, TerminalGuard};
use crate::tui;

/// Renders the current history command output from the stored transaction records.
pub async fn history(config: &mut Config, args: &History) -> Result<()> {
	let history_file = get_history(config).await?;

	if let Some(HistoryCommand::Undo(undo)) = &args.command {
		let entry = HistoryEntry::find_selector(&history_file, &undo.history_id)?;
		return entry.undo(config).await;
	}

	if let Some(HistoryCommand::Redo(redo)) = &args.command {
		let entry = HistoryEntry::find_selector(&history_file, &redo.history_id)?;
		return entry.redo(config).await;
	}

	let Some(history_id) = args.history_id.as_ref() else {
		if history_file.is_empty() {
			println!("No history entries found.");
			return Ok(());
		}

		println!("{}", HistoryEntry::list_table(&history_file));
		return Ok(());
	};

	let entry = HistoryEntry::find_selector(&history_file, &history_id)?;
	let pkg_set = entry.grouped_packages();

	if pkg_set.is_empty() {
		entry.print_detail(config);
		return Ok(());
	}

	if !use_tui(config) {
		entry.print_detail(config);
		return Ok(());
	}

	let cache = new_cache!()?;
	let mut terminal = TerminalGuard::new()?;
	tui::summary::SummaryTab::for_history(&cache, config, &pkg_set)
		.run(&mut terminal)
		.await?;

	Ok(())
}

#[cfg(test)]
mod tests;
