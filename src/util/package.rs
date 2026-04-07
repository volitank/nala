use rust_apt::records::RecordField;
use rust_apt::Version;

/// Return the package name. Checks if epoch is needed.
pub fn get_pkg_name(version: &Version) -> String {
	let filename = version
		.get_record(RecordField::Filename)
		.expect("Record does not contain a filename!")
		.split_terminator('/')
		.next_back()
		.expect("Filename is malformed!")
		.to_string();

	if let Some(index) = version.version().find(':') {
		let epoch = format!("_{}%3a", &version.version()[..index]);
		return filename.replacen('_', &epoch, 1);
	}
	filename
}
