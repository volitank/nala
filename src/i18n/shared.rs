use std::sync::OnceLock;

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentError, FluentResource};

static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();

const DEFAULT_LOCALE: &str = "en-US";
const EN_US: &str = include_str!("../../locales/en-US/main.ftl");

pub fn translate(
	id: &str,
	args: Option<&FluentArgs>,
	on_error: impl FnOnce(&[FluentError]),
) -> String {
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
		on_error(&errors);
		id.to_string()
	}
}

fn bundle() -> &'static FluentBundle<FluentResource> { BUNDLE.get_or_init(build_bundle) }

fn build_bundle() -> FluentBundle<FluentResource> {
	let resource =
		FluentResource::try_new(EN_US.to_string()).expect("bundled Fluent resource must parse");
	let locale = DEFAULT_LOCALE
		.parse()
		.expect("default Fluent locale must parse");
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
