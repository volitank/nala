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
		["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"]
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
	fn locale_names_are_recognized() {
		assert_eq!(Language::from_locale("bn_BD.UTF-8"), Language::Bn);
		assert_eq!(Language::from_locale("de_DE.UTF-8"), Language::De);
		assert_eq!(Language::from_locale("es_ES.UTF-8"), Language::Es);
		assert_eq!(Language::from_locale("fr_FR.UTF-8"), Language::Fr);
		assert_eq!(Language::from_locale("ga_IE.UTF-8"), Language::Ga);
		assert_eq!(Language::from_locale("pl_PL.UTF-8"), Language::Pl);
		assert_eq!(Language::from_locale("pt"), Language::Pt);
		assert_eq!(Language::from_locale("pt_BR.UTF-8"), Language::PtBr);
		assert_eq!(Language::from_locale("pt_PT.UTF-8"), Language::Pt);
		assert_eq!(Language::from_locale("ru_RU.UTF-8"), Language::Ru);
		assert_eq!(Language::from_locale("sv_SE.UTF-8"), Language::Sv);
		assert_eq!(Language::from_locale("tr_TR.UTF-8"), Language::Tr);
		assert_eq!(Language::from_locale("zh_CN.UTF-8"), Language::ZhCn);
		assert_eq!(Language::from_locale("fr:pt_BR:en"), Language::Fr);
		assert_eq!(Language::from_locale("xx:pt_BR:en"), Language::PtBr);
		assert_eq!(Language::from_locale("en:pt_BR"), Language::EnUs);
		assert_eq!(Language::from_locale("C.UTF-8"), Language::EnUs);
	}

	#[test]
	fn missing_localized_messages_fall_back_to_english() {
		assert_eq!(
			translate(Language::De, "history-empty", None),
			"Es existiert kein Verlauf."
		);
		assert_eq!(
			translate(Language::De, "history-clear-target", None),
			"History clear requires an entry selector or --all"
		);
	}

	#[test]
	fn portuguese_catalogs_are_distinct() {
		assert_eq!(
			translate(Language::Pt, "update-downloading", None),
			"A transferir"
		);
		assert_eq!(
			translate(Language::PtBr, "update-downloading", None),
			"Baixando"
		);
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
