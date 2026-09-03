use std::io::Write;

use anyhow::{Result, bail};

use crate::config::{Config, keys};
use crate::i18n::{Language, language};
use crate::t;

/// Ask the user for confirmation, honoring configured prompt defaults.
pub fn confirm(config: &Config, msg: &str) -> Result<()> {
	if confirm_with_default(config, msg, true)? {
		return Ok(());
	}

	bail!("{}", t!("prompt-refused"))
}

/// Ask the user a yes/no question with a configurable default.
pub fn confirm_with_default(config: &Config, msg: &str, default_yes: bool) -> Result<bool> {
	if config.get_bool(keys::ASSUME_NO, false) {
		return Ok(false);
	}

	if config.get_bool(keys::ASSUME_YES, false) {
		return Ok(true);
	}

	let choice = if default_yes { t!("prompt-choice") } else { t!("prompt-choice-no") };
	print!("{msg} {choice} ");
	std::io::stdout().flush()?;

	let mut response = String::new();
	if std::io::stdin().read_line(&mut response)? == 0 {
		return Ok(false);
	}

	response_answer(&response, default_yes)
		.ok_or_else(|| anyhow::anyhow!(t!("prompt-invalid", "response" => response.trim())))
}

fn response_answer(response: &str, default_yes: bool) -> Option<bool> {
	let response = response.trim().to_lowercase();
	if response.is_empty() {
		return Some(default_yes);
	}

	if response_is_yes(&response) {
		return Some(true);
	}

	response.starts_with('n').then_some(false)
}

fn response_is_yes(response: &str) -> bool {
	response.trim().is_empty()
		|| response.starts_with('y')
		|| (language() == Language::PtBr && response.starts_with('s'))
}

#[cfg(test)]
mod tests {
	use super::{confirm, response_answer, response_is_yes};
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

	#[test]
	fn confirmation_accepts_default_and_english_yes() {
		assert!(response_is_yes(""));
		assert!(response_is_yes("y"));
		assert!(!response_is_yes("n"));
	}

	#[test]
	fn yes_no_answer_uses_requested_default() {
		assert_eq!(response_answer("", true), Some(true));
		assert_eq!(response_answer("", false), Some(false));
		assert_eq!(response_answer("n", true), Some(false));
		assert_eq!(response_answer("wat", true), None);
	}
}
