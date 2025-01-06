use anyhow::{bail, Result};
use chrono::{DateTime, Local, Utc};
use rust_apt::new_cache;

use crate::config::Config;
use crate::libnala::get_history;
use crate::{table, tui};

pub async fn history(config: &Config) -> Result<()> {
	let history_file = get_history(config).await?;
	let cache = new_cache!()?;

	let mut table =
		table::get_table(&["ID", "Command", "Date and Time", "Requested-By", "Altered"]);

	// TODO: Make it configurable which timezones you want.
	// Convert Stored UTC into the local time zone
	let date_times = history_file
		.iter()
		.filter_map(|e| {
			Some(
				e.date
					.parse::<DateTime<Utc>>()
					.ok()?
					.with_timezone(&Local)
					.format("%Y-%m-%d %H:%M:%S %Z"),
			)
		})
		.collect::<Vec<_>>();

	for (i, entry) in history_file.iter().enumerate() {
		let row: Vec<&dyn std::fmt::Display> = vec![
			&entry.id,
			&entry.command,
			&date_times[i],
			&entry.requested_by,
			&entry.altered,
		];
		table.add_row(row);
	}

	if !config.get_no_bool("tui", true) {
		println!("{table}");
		return Ok(());
	}

	let num = 2;
	let Some(entry) = history_file.into_iter().nth(num - 1) else {
		bail!("History entry with ID '{num}' does not exist")
	};

	tui::summary::SummaryTab::new(&cache, config, &entry)
		.run()
		.await?;

	Ok(())
}
