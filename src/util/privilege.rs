use anyhow::{Result, bail};
use nix::unistd::Uid;

use crate::config::Config;

/// Check for root. Errors if not root.
/// Set up lock file if root.
pub fn sudo_check(config: &Config) -> Result<()> {
	if !Uid::effective().is_root() {
		bail!("Nala needs root to {}", config.command)
	}
	// TODO: Need to add lock file logic here maybe.
	Ok(())
}

/// Get the username or return Unknown.
pub(crate) fn get_user() -> (String, String) {
	let uid = std::env::var("SUDO_UID").unwrap_or_else(|_| Uid::effective().to_string());

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
