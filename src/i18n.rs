use std::sync::OnceLock;

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentResource};
use unic_langid::LanguageIdentifier;

static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();

const DEFAULT_LOCALE: &str = "en-US";
const EN_US: &str = include_str!("../locales/en-US/main.ftl");

pub fn translate(id: &str, args: Option<&FluentArgs>) -> String {
	let bundle = bundle();
	let Some(message) = bundle.get_message(id) else {
		return id.to_string();
	};

	let Some(pattern) = message.value() else {
		return id.to_string();
	};

	let mut errors = Vec::new();
	bundle
		.format_pattern(pattern, args, &mut errors)
		.into_owned()
}

fn bundle() -> &'static FluentBundle<FluentResource> {
	BUNDLE.get_or_init(|| build_bundle(current_locale(), EN_US))
}

fn build_bundle(locale: LanguageIdentifier, ftl: &str) -> FluentBundle<FluentResource> {
	let resource =
		FluentResource::try_new(ftl.to_string()).expect("bundled Fluent resource must parse");
	let mut bundle = FluentBundle::new_concurrent(vec![locale]);
	bundle.set_use_isolating(false);
	bundle
		.add_resource(resource)
		.expect("bundled Fluent messages must not conflict");
	bundle
}

fn current_locale() -> LanguageIdentifier {
	["NALA_LOCALE", "LC_ALL", "LC_MESSAGES", "LANG"]
		.into_iter()
		.filter_map(|key| std::env::var(key).ok())
		.find_map(|locale| parse_locale(&locale))
		.unwrap_or_else(default_locale)
}

fn parse_locale(locale: &str) -> Option<LanguageIdentifier> {
	let locale = locale
		.split(['.', '@'])
		.next()
		.unwrap_or(locale)
		.replace('_', "-");

	if locale.is_empty() || locale == "C" || locale == "POSIX" {
		return None;
	}

	locale.parse().ok()
}

fn default_locale() -> LanguageIdentifier {
	DEFAULT_LOCALE
		.parse()
		.expect("default Fluent locale must parse")
}

#[macro_export]
macro_rules! t {
	($id:expr) => {
		$crate::i18n::translate($id, None)
	};
	($id:expr, $($key:literal => $value:expr),+ $(,)?) => {{
		let mut args = ::fluent::FluentArgs::new();
		$(args.set($key, $value);)+
		$crate::i18n::translate($id, Some(&args))
	}};
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn locale_parser_accepts_posix_locale_names() {
		assert_eq!(parse_locale("en_US.UTF-8").unwrap().to_string(), "en-US");
		assert_eq!(
			parse_locale("pt_BR.UTF-8@latin").unwrap().to_string(),
			"pt-BR"
		);
	}

	#[test]
	fn locale_parser_rejects_process_default_locale_names() {
		assert!(parse_locale("C").is_none());
		assert!(parse_locale("POSIX").is_none());
	}

	#[test]
	fn translate_falls_back_to_message_id_when_missing() {
		assert_eq!(translate("missing-message-id", None), "missing-message-id");
	}

	#[test]
	fn translate_formats_plural_messages() {
		assert_eq!(
			crate::t!("history-cleared-count", "count" => 1),
			"Cleared 1 history entry."
		);
		assert_eq!(
			crate::t!("history-cleared-count", "count" => 2),
			"Cleared 2 history entries."
		);
	}
}
