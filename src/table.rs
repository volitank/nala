use comfy_table::Table;

use crate::config::Config;

pub fn get_table(config: &Config, headers: &[&str]) -> Table {
	let mut table = Table::new();
	table
		.load_preset(comfy_table::presets::NOTHING)
		.set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth)
		.set_header(headers.iter().map(|s| config.highlight(s)));

	table
		.column_mut(headers.len() - 1)
		.unwrap()
		.set_cell_alignment(comfy_table::CellAlignment::Right);

	table
}
