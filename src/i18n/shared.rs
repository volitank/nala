use std::sync::OnceLock;

use fluent::concurrent::FluentBundle;
use fluent::{FluentArgs, FluentError, FluentResource};

static EN_US_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static BN_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static DE_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static ES_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static FR_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static GA_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static PL_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static PT_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static PT_BR_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static RU_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static SV_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static TR_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
static ZH_CN_BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();

const EN_US: &str = include_str!("../../locales/en-US/main.ftl");
const BN: &str = include_str!("../../locales/bn/main.ftl");
const DE: &str = include_str!("../../locales/de/main.ftl");
const ES: &str = include_str!("../../locales/es/main.ftl");
const FR: &str = include_str!("../../locales/fr/main.ftl");
const GA: &str = include_str!("../../locales/ga/main.ftl");
const PL: &str = include_str!("../../locales/pl/main.ftl");
const PT: &str = include_str!("../../locales/pt/main.ftl");
const PT_BR: &str = include_str!("../../locales/pt-BR/main.ftl");
const RU: &str = include_str!("../../locales/ru/main.ftl");
const SV: &str = include_str!("../../locales/sv/main.ftl");
const TR: &str = include_str!("../../locales/tr/main.ftl");
const ZH_CN: &str = include_str!("../../locales/zh-CN/main.ftl");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
	Bn,
	De,
	EnUs,
	Es,
	Fr,
	Ga,
	Pl,
	Pt,
	PtBr,
	Ru,
	Sv,
	Tr,
	ZhCn,
}

const LANGUAGE_ALIASES: &[(&[&str], Language)] = &[
	(&["bn", "bn_bd", "bn-bd"], Language::Bn),
	(&["de", "de_de", "de-de"], Language::De),
	(&["en", "en_us", "en-us", "c", "posix"], Language::EnUs),
	(&["es", "es_es", "es-es"], Language::Es),
	(&["fr", "fr_fr", "fr-fr"], Language::Fr),
	(&["ga", "ga_ie", "ga-ie"], Language::Ga),
	(&["pl", "pl_pl", "pl-pl"], Language::Pl),
	(&["pt", "pt_pt", "pt-pt"], Language::Pt),
	(&["pt_br", "pt-br"], Language::PtBr),
	(&["ru", "ru_ru", "ru-ru"], Language::Ru),
	(&["sv", "sv_se", "sv-se"], Language::Sv),
	(&["tr", "tr_tr", "tr-tr"], Language::Tr),
	(&["zh", "zh_cn", "zh-cn"], Language::ZhCn),
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
		Language::Bn => BN_BUNDLE.get_or_init(|| build_bundle("bn", Some(BN))),
		Language::De => DE_BUNDLE.get_or_init(|| build_bundle("de", Some(DE))),
		Language::EnUs => EN_US_BUNDLE.get_or_init(|| build_bundle("en-US", None)),
		Language::Es => ES_BUNDLE.get_or_init(|| build_bundle("es", Some(ES))),
		Language::Fr => FR_BUNDLE.get_or_init(|| build_bundle("fr", Some(FR))),
		Language::Ga => GA_BUNDLE.get_or_init(|| build_bundle("Ga", Some(GA))),
		Language::Pl => PL_BUNDLE.get_or_init(|| build_bundle("pl", Some(PL))),
		Language::Pt => PT_BUNDLE.get_or_init(|| build_bundle("pt-PT", Some(PT))),
		Language::PtBr => PT_BR_BUNDLE.get_or_init(|| build_bundle("pt-BR", Some(PT_BR))),
		Language::Ru => RU_BUNDLE.get_or_init(|| build_bundle("ru", Some(RU))),
		Language::Sv => SV_BUNDLE.get_or_init(|| build_bundle("sv", Some(SV))),
		Language::Tr => TR_BUNDLE.get_or_init(|| build_bundle("tr", Some(TR))),
		Language::ZhCn => ZH_CN_BUNDLE.get_or_init(|| build_bundle("zh-CN", Some(ZH_CN))),
	}
}

fn build_bundle(locale: &str, source: Option<&str>) -> FluentBundle<FluentResource> {
	let locale = locale.parse().expect("bundled Fluent locale must parse");
	let mut bundle = FluentBundle::new_concurrent(vec![locale]);
	bundle
		.add_resource(parse_resource(EN_US))
		.expect("bundled Fluent messages must not conflict");

	if let Some(source) = source {
		bundle.add_resource_overriding(parse_resource(source));
	}

	bundle
}

fn parse_resource(source: &str) -> FluentResource {
	FluentResource::try_new(source.to_string()).expect("bundled Fluent resource must parse")
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

	use super::{BN, DE, EN_US, ES, FR, GA, Language, PL, PT, PT_BR, RU, SV, TR, ZH_CN, bundle};

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
	fn localized_catalogs_only_override_english_messages() {
		let english = message_ids(EN_US);

		for source in [BN, DE, ES, FR, GA, PL, PT, PT_BR, RU, SV, TR, ZH_CN] {
			assert!(message_ids(source).is_subset(&english));
		}
	}

	#[test]
	fn every_catalog_builds() {
		for language in [
			Language::Bn,
			Language::De,
			Language::EnUs,
			Language::Es,
			Language::Fr,
			Language::Ga,
			Language::Pl,
			Language::Pt,
			Language::PtBr,
			Language::Ru,
			Language::Sv,
			Language::Tr,
			Language::ZhCn,
		] {
			assert!(bundle(language).get_message("summary-title").is_some());
		}
	}
}
