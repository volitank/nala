use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Local, Utc};

use super::model::HistoryEntry;
use crate::cmd::Operation;
use crate::config::Config;
use crate::libnala::PackageTransition;
use crate::tui::summary::SummaryRow;
use crate::{t, table};

impl HistoryEntry {
	/// Formats a stored UTC timestamp for local display, or returns the raw value.
	pub(super) fn format_timestamp(timestamp: &str) -> String {
		timestamp
			.parse::<DateTime<Utc>>()
			.map(|date| {
				date.with_timezone(&Local)
					.format("%Y-%m-%d %H:%M:%S %Z")
					.to_string()
			})
			.unwrap_or_else(|_| timestamp.to_string())
	}

	/// Returns the entry start time formatted for display.
	fn started_at_display(&self) -> String {
		Self::format_timestamp(&self.started_at)
	}

	/// Returns the entry finish time formatted for display.
	fn finished_at_display(&self) -> String {
		Self::format_timestamp(&self.finished_at)
	}

	/// Builds the plain history list table from stored entries.
	pub(super) fn list_table(entries: &[Self]) -> comfy_table::Table {
		let headers = [
			t!("history-table-id"),
			t!("history-table-command"),
			t!("history-table-date-time"),
			t!("history-table-requested-by"),
			t!("history-table-altered"),
		];
		let header_refs = headers.iter().map(String::as_str).collect::<Vec<_>>();
		let mut table = table::get_table(&header_refs);

		for entry in entries {
			let date_time = entry.started_at_display();
			let altered = entry.altered().count();
			let row: Vec<&dyn std::fmt::Display> = vec![
				&entry.id,
				&entry.command,
				&date_time,
				&entry.requested_by,
				&altered,
			];
			table.add_row(row);
		}

		table
	}

	/// Regroups the stored package rows by operation for display.
	pub(super) fn grouped_packages(&self) -> HashMap<Operation, Vec<&PackageTransition>> {
		let mut pkg_set: HashMap<Operation, Vec<&PackageTransition>> = HashMap::new();

		for pkg in &self.packages {
			pkg_set.entry(pkg.operation).or_default().push(pkg);
		}

		pkg_set
	}

	/// Selects a stored history entry by its durable transaction ID.
	pub(super) fn find(entries: &[Self], id: u32) -> Result<&Self> {
		entries
			.iter()
			.find(|entry| entry.id == id)
			.ok_or_else(|| anyhow!(t!("history-entry-not-found", "id" => id)))
	}

	/// Prints a plain-text detail view for this history entry.
	pub(super) fn print_detail(&self, config: &Config) {
		let requested_targets = if self.requested_targets.is_empty() {
			t!("history-detail-none")
		} else {
			self.requested_targets.join(", ")
		};

		println!("{}: {}", t!("history-detail-id"), self.id);
		println!("{}: {:?}", t!("history-detail-status"), self.status);
		println!("{}: {}", t!("history-detail-command"), self.command);
		println!(
			"{}: {}",
			t!("history-detail-requested-by"),
			self.requested_by
		);
		println!(
			"{}: {}",
			t!("history-detail-started"),
			self.started_at_display()
		);
		println!(
			"{}: {}",
			t!("history-detail-finished"),
			self.finished_at_display()
		);
		println!(
			"{}: {requested_targets}",
			t!("history-detail-requested-targets")
		);
		println!("{}: {}", t!("history-detail-altered"), self.altered().count());

		let pkg_set = self.grouped_packages();
		if pkg_set.is_empty() {
			return;
		}

		for operation in Operation::to_vec() {
			let Some(packages) = pkg_set.get(&operation) else {
				continue;
			};
			let rows = packages
				.iter()
				.map(|package| SummaryRow::new(package))
				.collect::<Vec<_>>();

			println!();
			println!("{} ({})", operation, packages.len());

			let mut table = table::get_table(&rows[0].headers());

			table.add_rows(rows.iter().map(|row| row.items(config)));
			println!("{table}");
		}
	}
}
