use std::sync::OnceLock;

use fluent::FluentArgs;

pub(crate) use self::shared::Language;

static LANGUAGE: OnceLock<Language> = OnceLock::new();

pub fn translate(id: &str, args: Option<&FluentArgs>) -> String {
	shared::translate(language(), id, args, |errors| {
		crate::debug!("Failed to format Fluent message '{id}': {errors:?}");
	})
}

pub(crate) fn language() -> Language {
	*LANGUAGE.get_or_init(|| {
		["LC_ALL", "LC_MESSAGES", "LANGUAGE", "LANG"]
			.into_iter()
			.find_map(|name| std::env::var(name).ok().filter(|locale| !locale.is_empty()))
			.map_or(Language::EnUs, |locale| Language::from_locale(&locale))
	})
}

mod shared;

#[cfg(test)]
mod tests {
	use super::*;

	fn translate(language: Language, id: &str, args: Option<&FluentArgs>) -> String {
		shared::translate(language, id, args, |_| {})
	}

	#[test]
	fn translate_falls_back_to_message_id_when_missing() {
		assert_eq!(
			translate(Language::EnUs, "missing-message-id", None),
			"missing-message-id"
		);
	}

	#[test]
	fn translate_falls_back_to_message_id_when_formatting_fails() {
		assert_eq!(
			translate(Language::EnUs, "history-cleared", None),
			"history-cleared"
		);
	}

	#[test]
	fn translate_formats_plural_messages_with_isolating() {
		let mut args = FluentArgs::new();

		args.set("count", 0);
		assert_eq!(
			translate(Language::EnUs, "history-cleared", Some(&args)),
			"Cleared \u{2068}0\u{2069} history entries."
		);

		args.set("count", 1);
		assert_eq!(
			translate(Language::EnUs, "history-cleared", Some(&args)),
			"Cleared \u{2068}1\u{2069} history entry."
		);

		args.set("count", 2);
		assert_eq!(
			translate(Language::EnUs, "history-cleared", Some(&args)),
			"Cleared \u{2068}2\u{2069} history entries."
		);
	}

	#[test]
	fn brazilian_portuguese_locale_names_are_recognized() {
		assert_eq!(Language::from_locale("pt_BR.UTF-8"), Language::PtBr);
		assert_eq!(Language::from_locale("pt-BR"), Language::PtBr);
		assert_eq!(Language::from_locale("pt_BR:en"), Language::PtBr);
		assert_eq!(Language::from_locale("C.UTF-8"), Language::EnUs);
	}

	#[test]
	fn brazilian_portuguese_messages_are_formatted() {
		let mut args = FluentArgs::new();
		args.set("count", 2);

		assert_eq!(
			translate(Language::PtBr, "history-cleared", Some(&args)),
			"Limpou \u{2068}2\u{2069} entradas do histórico."
		);
	}
}
