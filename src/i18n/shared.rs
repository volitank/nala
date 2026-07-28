use std::sync::OnceLock;

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentError, FluentResource};

static EN_US_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static PT_BR_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();

const EN_US: &str = include_str!("../../locales/en-US/main.ftl");
const PT_BR: &str = include_str!("../../locales/pt-BR/main.ftl");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
	EnUs,
	PtBr,
}

const LANGUAGE_ALIASES: &[(&[&str], Language)] = &[
	(&["en", "en_us", "en-us", "c", "posix"], Language::EnUs),
	(&["pt", "pt_br", "pt-br"], Language::PtBr),
];

impl Language {
	pub fn from_locale(locale: &str) -> Self {
		for locale in locale.split(':') {
			let locale = locale.split(['.', '@']).next().unwrap_or(locale);
			for (aliases, language) in LANGUAGE_ALIASES {
				if aliases
					.iter()
					.any(|alias| locale.eq_ignore_ascii_case(alias))
				{
					return *language;
				}
			}
		}

		Self::EnUs
	}
}

pub fn translate(
	language: Language,
	id: &str,
	args: Option<&FluentArgs>,
	on_error: impl FnOnce(&[FluentError]),
) -> String {
	let bundle = bundle(language);
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
		on_error(&errors);
		id.to_string()
	}
}

fn bundle(language: Language) -> &'static FluentBundle<FluentResource> {
	match language {
		Language::EnUs => EN_US_BUNDLE.get_or_init(|| build_bundle(EN_US, "en-US")),
		Language::PtBr => PT_BR_BUNDLE.get_or_init(|| build_bundle(PT_BR, "pt-BR")),
	}
}

fn build_bundle(source: &str, locale: &str) -> FluentBundle<FluentResource> {
	let resource =
		FluentResource::try_new(source.to_string()).expect("bundled Fluent resource must parse");
	let locale = locale.parse().expect("bundled Fluent locale must parse");
	let mut bundle = FluentBundle::new_concurrent(vec![locale]);
	bundle
		.add_resource(resource)
		.expect("bundled Fluent messages must not conflict");
	bundle
}

#[macro_export]
macro_rules! t {
	($id:literal) => {
		$crate::i18n::translate($id, None)
	};
	($id:literal, $($key:literal => $value:expr),+ $(,)?) => {{
		let mut args = ::fluent::FluentArgs::new();
		$(args.set($key, $value);)+
		$crate::i18n::translate($id, Some(&args))
	}};
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeSet;

	use super::{EN_US, PT_BR};

	fn message_ids(source: &str) -> BTreeSet<&str> {
		source
			.lines()
			.filter_map(|line| {
				let (id, _) = line.split_once(" =")?;
				id.chars()
					.all(|char| char.is_ascii_lowercase() || char.is_ascii_digit() || char == '-')
					.then_some(id)
			})
			.collect()
	}

	#[test]
	fn catalogs_have_the_same_messages() { assert_eq!(message_ids(EN_US), message_ids(PT_BR)) }
}
