use std::io::Write;

use anyhow::{Result, bail};

use crate::config::{Config, keys};
use crate::t;

/// Ask the user for confirmation, honoring configured prompt defaults.
pub fn confirm(config: &Config, msg: &str) -> Result<()> {
	if config.get_bool(keys::ASSUME_NO, false) {
		bail!("{}", t!("prompt-refused"));
	}

	if config.get_bool(keys::ASSUME_YES, false) {
		return Ok(());
	}

	print!("{msg} {} ", t!("prompt-choice"));
	std::io::stdout().flush()?;

	let mut response = String::new();
	std::io::stdin().read_line(&mut response)?;

	let resp = response.to_lowercase();
	if resp.trim().is_empty() || resp.starts_with('y') {
		return Ok(());
	}

	if resp.starts_with('n') {
		bail!("{}", t!("prompt-refused"))
	}

	bail!("{}", t!("prompt-invalid", "response" => response.trim()))
}

#[cfg(test)]
mod tests {
	use super::confirm;
	use crate::config::{Config, keys};

	#[test]
	fn confirm_honors_assume_yes_without_prompting() {
		let mut config = Config::default();
		config.set_bool(keys::ASSUME_YES, true);

		assert!(confirm(&config, "Continue?").is_ok());
	}

	#[test]
	fn confirm_honors_assume_no_before_assume_yes() {
		let mut config = Config::default();
		config.set_bool(keys::ASSUME_YES, true);
		config.set_bool(keys::ASSUME_NO, true);

		assert!(confirm(&config, "Continue?").is_err());
	}
}
