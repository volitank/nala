use std::sync::OnceLock;

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentResource};

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
	let translated = bundle
		.format_pattern(pattern, args, &mut errors)
		.into_owned();

	if errors.is_empty() {
		translated
	} else {
		crate::debug!("Failed to format Fluent message '{id}': {errors:?}");
		id.to_string()
	}
}

fn bundle() -> &'static FluentBundle<FluentResource> { BUNDLE.get_or_init(|| build_bundle(EN_US)) }

fn build_bundle(ftl: &str) -> FluentBundle<FluentResource> {
	let resource =
		FluentResource::try_new(ftl.to_string()).expect("bundled Fluent resource must parse");
	let locale = DEFAULT_LOCALE
		.parse()
		.expect("default Fluent locale must parse");
	let mut bundle = FluentBundle::new_concurrent(vec![locale]);
	bundle.set_use_isolating(false);
	bundle
		.add_resource(resource)
		.expect("bundled Fluent messages must not conflict");
	bundle
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
	fn translate_falls_back_to_message_id_when_missing() {
		assert_eq!(translate("missing-message-id", None), "missing-message-id");
	}

	#[test]
	fn translate_falls_back_to_message_id_when_formatting_fails() {
		assert_eq!(
			translate("history-cleared-count", None),
			"history-cleared-count"
		);
	}

	#[test]
	fn translate_formats_plural_messages() {
		assert_eq!(
			crate::t!("history-cleared-count", "count" => 0),
			"Cleared 0 history entries."
		);
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
