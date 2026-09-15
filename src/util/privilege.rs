use anyhow::{Result, bail};
use nix::unistd::Uid;

use crate::config::Config;
use crate::t;

/// Holds APT's frontend lock until the current mutation finishes.
pub(crate) struct AptLockGuard;

impl AptLockGuard {
	pub(crate) fn acquire() -> Result<Self> {
		rust_apt::util::apt_lock()?;
		Ok(Self)
	}
}

impl Drop for AptLockGuard {
	fn drop(&mut self) { rust_apt::util::apt_unlock(); }
}

/// Check for root. Errors if not root.
pub fn sudo_check(config: &Config) -> Result<()> {
	if !Uid::effective().is_root() {
		bail!("{}", t!("root-required", "command" => &config.command))
	}
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
		t!("unknown")
	});

	(uid, username)
}
