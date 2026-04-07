use crate::config::{color, Theme};

pub fn version_diff(old: &str, new: String) -> String {
	// Check for just revision change first.
	if let (Some(old_ver), Some(new_ver)) = (old.rsplit_once('-'), new.rsplit_once('-')) {
		// If there isn't a revision these shouldn't ever match
		// If they do match then only the revision has changed
		if old_ver.0 == new_ver.0 {
			return format!("{}-{}", new_ver.0, color::color!(Theme::Notice, new_ver.0));
		}
	}

	let (old_ver, new_ver) = (
		old.split('.').collect::<Vec<_>>(),
		new.split('.').collect::<Vec<_>>(),
	);

	let mut start_color = 0;
	for (i, section) in old_ver.iter().enumerate() {
		if i > new_ver.len() - 1 {
			break;
		}

		if section != &new_ver[i] {
			start_color = i;
			break;
		}
	}

	new_ver
		.iter()
		.enumerate()
		.map(|(i, str)| {
			if i >= start_color {
				color::color!(Theme::Notice, str).to_string()
			} else {
				str.to_string()
			}
		})
		.collect::<Vec<_>>()
		.join(".")
}
