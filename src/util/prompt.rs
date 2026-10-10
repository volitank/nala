use std::io::Write;

use anyhow::{Result, bail};

use crate::config::{Config, keys};
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

	let keys = confirmation_keys(&t!("prompt-choice"));
	let choice = confirmation_choice(keys, default_yes);
	print!("{msg} {choice} ");
	std::io::stdout().flush()?;

	let mut response = String::new();
	if std::io::stdin().read_line(&mut response)? == 0 {
		return Ok(false);
	}

	response_answer(&response, default_yes, keys)
		.ok_or_else(|| anyhow::anyhow!(t!("prompt-invalid", "response" => response.trim())))
}

pub(crate) fn confirmation_keys(choice: &str) -> (char, char) {
	let english = ('y', 'n');
	let chars: Vec<char> = choice.to_lowercase().chars().collect();

	let [yes, '/', repeated_yes, ' ', no, '/', repeated_no] = chars.as_slice() else {
		return english;
	};

	let forms_match = yes == repeated_yes && no == repeated_no;
	let keys_are_letters = yes.is_alphabetic() && no.is_alphabetic();

	// Translated keys must not contradict the English y/n answers.
	let keys_are_unambiguous = yes != no && *yes != 'n' && *no != 'y';

	if forms_match && keys_are_letters && keys_are_unambiguous {
		(*yes, *no)
	} else {
		english
	}
}

pub(crate) fn confirmation_choice((yes, no): (char, char), default_yes: bool) -> String {
	if default_yes {
		format!("[{}/{no}]", yes.to_uppercase())
	} else {
		format!("[{yes}/{}]", no.to_uppercase())
	}
}

pub(crate) fn response_answer(
	response: &str,
	default_yes: bool,
	(yes, no): (char, char),
) -> Option<bool> {
	let response = response.trim().to_lowercase();
	if response.is_empty() {
		return Some(default_yes);
	}

	match response.chars().next()? {
		'n' => Some(false),
		'y' => Some(true),
		answer if answer == no => Some(false),
		answer if answer == yes => Some(true),
		_ => None,
	}
}

#[cfg(test)]
mod tests {
	use super::{confirm, confirmation_choice, confirmation_keys, response_answer};
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
	fn translated_keys_drive_hints_and_answers() {
		for (choice, yes, no) in [
			("Y/y N/n", 'y', 'n'),
			("J/j N/n", 'j', 'n'),
			("S/s N/n", 's', 'n'),
			("O/o N/n", 'o', 'n'),
			("T/t N/n", 't', 'n'),
			("Д/д Н/н", 'д', 'н'),
			("E/e H/h", 'e', 'h'),
		] {
			let keys = confirmation_keys(choice);
			assert_eq!(keys, (yes, no));
			assert_eq!(
				confirmation_choice(keys, true),
				format!("[{}/{no}]", yes.to_uppercase())
			);
			assert_eq!(
				confirmation_choice(keys, false),
				format!("[{yes}/{}]", no.to_uppercase())
			);
			for response in [
				yes.to_string(),
				yes.to_uppercase().to_string(),
				"y".into(),
				"Y".into(),
			] {
				assert_eq!(response_answer(&response, false, keys), Some(true));
			}
			for response in [
				no.to_string(),
				no.to_uppercase().to_string(),
				"n".into(),
				"N".into(),
			] {
				assert_eq!(response_answer(&response, true, keys), Some(false));
			}
			assert_eq!(response_answer("", true, keys), Some(true));
			assert_eq!(response_answer("", false, keys), Some(false));
			assert_eq!(response_answer("wat", true, keys), None);
		}
	}

	#[test]
	fn malformed_or_conflicting_translated_keys_use_english() {
		for choice in [
			"", "[Y/n]", "Y/y", "Y/y Y/y", "N/n Y/y", "J/x N/n", "1/1 N/n",
		] {
			assert_eq!(confirmation_keys(choice), ('y', 'n'));
		}
	}
}
