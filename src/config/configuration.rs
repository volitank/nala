use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Result};
use clap::parser::ValueSource;
use clap::ArgMatches;
use rust_apt::config::Config as AptConfig;

use super::color::setup_color;
use super::file::{ConfigFile, UiMode};
use super::paths::PathSpec;
use super::{keys, OptType, Paths, Switch};
use crate::config::color::{Style, Theme};
use crate::util::UnitStr;

#[derive(Debug)]
pub struct Config {
	file: ConfigFile,
	overrides: HashMap<String, OptType>,
	pub apt: AptConfig,
	pub command: String,
}

impl Default for Config {
	fn default() -> Self { Self::from_file(ConfigFile::default()) }
}

impl Config {
	pub fn new(conf_file: &Path) -> Result<Self> {
		let file = ConfigFile::read(conf_file)?;
		Ok(Self::from_file(file))
	}

	pub fn from_file(file: ConfigFile) -> Self {
		Self {
			file,
			overrides: HashMap::new(),
			apt: AptConfig::new(),
			command: "Command Not Given Yet".to_string(),
		}
	}

	pub fn file(&self) -> &ConfigFile { &self.file }

	pub fn style<T: AsRef<Theme>>(&self, theme: T) -> &Style { self.file.color.style(theme) }

	pub fn load_args(&mut self, args: &ArgMatches) -> Result<()> {
		for alias in [
			("full-upgrade", keys::FULL),
			("safe-upgrade", keys::SAFE),
			("autopurge", keys::PURGE),
			("purge", keys::PURGE),
		] {
			if std::env::args().any(|arg| arg == alias.0) {
				self.overrides
					.insert(alias.1.to_string(), OptType::Bool(true));
			}
		}

		for id in args.ids() {
			let key = id.as_str().to_string();
			if Some(ValueSource::CommandLine) != args.value_source(&key) {
				continue;
			}

			if let Ok(Some(value)) = args.try_get_one::<bool>(&key) {
				self.overrides.insert(key, OptType::Bool(*value));
				continue;
			}

			if let Ok(Some(value)) = args.try_get_occurrences::<String>(&key) {
				self.overrides
					.insert(key, OptType::VecString(value.flatten().cloned().collect()));
				continue;
			}

			if let Ok(Some(value)) = args.try_get_one::<String>(&key) {
				self.overrides.insert(key, OptType::String(value.clone()));
				continue;
			}

			if let Ok(Some(value)) = args.try_get_one::<u8>(&key) {
				self.overrides.insert(key, OptType::Int(*value));
				continue;
			}

			if let Ok(Some(value)) = args.try_get_one::<u64>(&key) {
				self.overrides.insert(key, OptType::Int64(*value));
			}
		}

		setup_color(self.file.color.with_mode(self.color_mode()));

		if let Some(options) = self.get_vec(keys::OPTION) {
			for raw_opt in options {
				let Some((key, value)) = raw_opt.split_once("=") else {
					bail!("Option '{raw_opt}' is not supported");
				};
				self.apt.set(key, value);
			}
		}

		Ok(())
	}

	pub fn get_bool(&self, key: &str, default: bool) -> bool {
		self.overrides
			.get(key)
			.and_then(OptType::as_bool)
			.or_else(|| self.file.bool(key))
			.unwrap_or(default)
	}

	pub fn set_bool(&mut self, key: &str, value: bool) {
		self.overrides.insert(key.to_string(), OptType::Bool(value));
	}

	pub fn set_history_dir<S: Into<String>>(&mut self, dir: S) {
		self.overrides
			.insert(keys::HISTORY_DIR.to_string(), OptType::String(dir.into()));
	}

	pub fn get_str(&self, key: &str) -> Option<&str> {
		self.overrides.get(key).and_then(|opt| {
			opt.as_vec_string()
				.and_then(|v| v.first().map(|s| s.as_str()))
				.or(opt.as_string())
		})
	}

	pub fn get_vec(&self, key: &str) -> Option<&Vec<String>> {
		self.overrides.get(key).and_then(OptType::as_vec_string)
	}

	pub fn get_mut_vec(&mut self, key: &str) -> Option<&mut Vec<String>> {
		if let OptType::VecString(vec) = self.overrides.get_mut(key)? {
			return Some(vec);
		}
		None
	}

	pub fn get_path(&self, dir: &Paths) -> PathBuf {
		match dir.spec() {
			PathSpec::Fixed(path) if matches!(dir, Paths::History) => self
				.get_str(keys::HISTORY_DIR)
				.map(PathBuf::from)
				.unwrap_or_else(|| PathBuf::from(path)),
			PathSpec::Fixed(path) => PathBuf::from(path),
			PathSpec::Apt { key, default } => PathBuf::from(self.apt.file(key, default)),
		}
	}

	pub fn get_file(&self, file: &Paths) -> String { self.get_path(file).to_string_lossy().into() }

	pub fn ui_mode(&self) -> UiMode {
		if self.get_bool(keys::NO_TUI, false) {
			return UiMode::Plain;
		}

		if self.get_bool(keys::TUI, false) {
			return UiMode::Tui;
		}

		self.file.ui.mode
	}

	pub fn color_mode(&self) -> Switch {
		self.overrides
			.get(keys::COLOR)
			.and_then(OptType::as_switch)
			.unwrap_or_else(|| self.file.color_mode())
	}

	pub fn unit_format(&self) -> UnitStr { self.file.unit_format() }

	pub fn unit_str(&self, unit: u64) -> String { self.unit_format().str(unit) }

	pub fn get_no_bool(&self, key: &str, default: bool) -> bool {
		let mut no_option = String::from("no_");
		no_option += key;
		if self.get_bool(&no_option, false) {
			return false;
		}
		self.get_bool(key, default)
	}

	pub fn pkg_names(&self) -> Result<Vec<String>> {
		let Some(pkg_names) = self.get_vec(keys::PKG_NAMES) else {
			bail!("You must specify a package");
		};

		let mut deduped = pkg_names.clone();
		deduped.sort();
		deduped.dedup();

		Ok(deduped)
	}

	pub fn arches(&self) -> Vec<String> {
		if self.get_bool(keys::ALL_ARCHES, false) {
			self.apt.get_architectures()
		} else {
			vec![self.apt.get_architectures().into_iter().next().unwrap()]
		}
	}

	pub fn countries(&self) -> Option<&Vec<String>> { self.get_vec(keys::COUNTRY) }

	pub fn auto(&self) -> Option<u8> { self.overrides.get(keys::AUTO).and_then(OptType::as_int) }

	pub fn allow_unauthenticated(&self) -> bool {
		self.get_bool(keys::ALLOW_UNAUTHENTICATED, false)
			|| self.apt.bool("APT::Get::AllowUnauthenticated", false)
	}

	pub fn debug(&self) -> bool { self.get_bool(keys::DEBUG, false) }

	pub fn verbose(&self) -> bool { self.get_bool(keys::VERBOSE, self.debug()) }
}

#[cfg(test)]
mod test {
	use std::sync::{LazyLock, Mutex, MutexGuard};

	use clap::CommandFactory;

	use super::Config;
	use crate::cli::NalaParser;
	use crate::config::file::{ConfigFile, UiMode};
	use crate::config::{keys, OptType, Paths, Switch};
	use crate::util::NumSys;

	static TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

	fn test_lock() -> MutexGuard<'static, ()> { TEST_LOCK.lock().unwrap() }

	#[test]
	fn file_defaults_match_the_target_shape() {
		let _guard = test_lock();
		let file = ConfigFile::default();

		assert!(file.nala.auto_remove);
		assert!(file.nala.auto_update);
		assert!(!file.nala.update_show_packages);
		assert_eq!(file.ui.mode, UiMode::Auto);
		assert_eq!(file.ui.unit, NumSys::Binary);
		assert_eq!(file.color.mode, Switch::Auto);
	}

	#[test]
	fn fixed_paths_are_constant() {
		let _guard = test_lock();
		let config = Config::default();

		let nala_sources = config.get_path(&Paths::NalaSources);
		assert_eq!(
			nala_sources.to_string_lossy(),
			"/etc/apt/sources.list.d/nala.sources"
		);

		let history = config.get_path(&Paths::History);
		assert_eq!(history.to_string_lossy(), "/var/lib/nala/history");
	}

	#[test]
	fn history_dir_override_changes_history_path_only() {
		let _guard = test_lock();
		let mut config = Config::default();

		config.set_history_dir("/tmp/nala-history-test");

		assert_eq!(
			config.get_path(&Paths::History).to_string_lossy(),
			"/tmp/nala-history-test"
		);
		assert_eq!(
			config.get_path(&Paths::NalaSources).to_string_lossy(),
			"/etc/apt/sources.list.d/nala.sources"
		);
	}

	#[test]
	fn pkg_names_are_sorted_and_deduped() {
		let _guard = test_lock();
		let mut config = Config::default();
		config.overrides.insert(
			keys::PKG_NAMES.to_string(),
			OptType::VecString(vec!["b".into(), "a".into(), "b".into(), "a".into()]),
		);

		let names = config.pkg_names().unwrap();
		assert_eq!(names, vec!["a", "b"]);
	}

	#[test]
	fn no_bool_overrides_true() {
		let _guard = test_lock();
		let mut config = Config::default();
		config.set_bool("feature", true);
		config.set_bool("no_feature", true);

		assert!(!config.get_no_bool("feature", false));
	}

	#[test]
	fn assume_prompt_flags_load_from_cli() {
		let _guard = test_lock();
		let args = NalaParser::command()
			.try_get_matches_from(["nala", "install", "--assume-no", "demo"])
			.unwrap();
		let (_, cmd) = args.subcommand().unwrap();
		let mut config = Config::default();

		config.load_args(cmd).unwrap();

		assert!(config.get_bool(keys::ASSUME_NO, false));
	}

	#[test]
	fn assume_yes_uses_config_default() {
		let _guard = test_lock();
		let mut file = ConfigFile::default();
		file.nala.assume_yes = true;
		let config = Config::from_file(file);

		assert!(config.get_bool(keys::ASSUME_YES, false));
	}

	#[test]
	fn unit_format_uses_ui_section() {
		let _guard = test_lock();
		let mut file = ConfigFile::default();
		file.ui.unit = NumSys::Decimal;

		let config = Config::from_file(file);
		assert_eq!(config.unit_str(1_024), "1 KB");
	}

	#[test]
	fn ui_mode_respects_runtime_overrides() {
		let _guard = test_lock();
		let mut config = Config::default();

		assert_eq!(config.ui_mode(), UiMode::Auto);

		config.set_bool(keys::NO_TUI, true);
		assert_eq!(config.ui_mode(), UiMode::Plain);

		config.set_bool(keys::NO_TUI, false);
		config.set_bool(keys::TUI, true);
		assert_eq!(config.ui_mode(), UiMode::Tui);
	}
}
