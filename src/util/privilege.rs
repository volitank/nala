use anyhow::{bail, Result};

use crate::config::Config;

#[link(name = "c")]
extern "C" {
	fn geteuid() -> u32;
}

/// Check for root. Errors if not root.
/// Set up lock file if root.
pub fn sudo_check(config: &Config) -> Result<()> {
	if unsafe { geteuid() != 0 } {
		bail!("Nala needs root to {}", config.command)
	}
	// TODO: Need to add lock file logic here maybe.
	Ok(())
}

/// Get the username or return Unknown.
pub(crate) fn get_user() -> (String, String) {
	let uid = std::env::var("SUDO_UID").unwrap_or_else(|_| format!("{}", unsafe { geteuid() }));

	let username = std::env::var("SUDO_USER").unwrap_or_else(|_| {
		for key in ["LOGNAME", "USER", "LNAME", "USERNAME"] {
			if let Ok(name) = std::env::var(key) {
				return name;
			}
		}
		"Unknown".to_string()
	});

	(uid, username)
}
