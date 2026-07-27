use comfy_table::Table;

use crate::config::color;

pub fn get_table<S: AsRef<str>>(headers: &[S]) -> Table {
	let mut table = Table::new();
	table
		.load_preset(comfy_table::presets::NOTHING)
		.set_content_arrangement(comfy_table::ContentArrangement::DynamicFullWidth)
		.set_header(headers.iter().map(|s| color::highlight!(s.as_ref())));

	table
		.column_mut(headers.len() - 1)
		.unwrap()
		.set_cell_alignment(comfy_table::CellAlignment::Right);

	table
}
