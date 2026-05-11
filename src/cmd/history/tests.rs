use super::replay::ReplayAction;
use super::*;
use super::model::{HistoryStatus, HISTORY_SCHEMA_VERSION};
use crate::cli::HistorySelector;
use crate::config::Config;
use crate::libnala::{Operation, PackageState, PackageTransition};
use std::fs;
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

static HISTORY_STORE_TEST_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

fn history_store_test_lock() -> MutexGuard<'static, ()> {
	HISTORY_STORE_TEST_LOCK.lock().unwrap()
}

#[test]
fn entry_records_requested_targets_and_status() {
	let config = Config::default();
	let entry = HistoryEntry::applied(
		&config,
		7,
		"2026-04-11T00:00:00Z".to_string(),
		"2026-04-11T00:01:00Z".to_string(),
		vec![],
	);

	assert_eq!(entry.schema_version, HISTORY_SCHEMA_VERSION);
	assert_eq!(entry.id, 7);
	assert_eq!(entry.status, HistoryStatus::Applied);
	assert!(entry.requested_targets.is_empty());
}

#[test]
fn history_entry_selects_by_recorded_id() {
	let entries = vec![
		HistoryEntry {
			schema_version: HISTORY_SCHEMA_VERSION,
			id: 4,
			started_at: "2026-04-11T00:00:00Z".to_string(),
			finished_at: "2026-04-11T00:01:00Z".to_string(),
			status: HistoryStatus::Applied,
			requested_by: "user (1000)".to_string(),
			command: "install a".to_string(),
			requested_targets: vec!["a".to_string()],
			packages: vec![],
		},
		HistoryEntry {
			schema_version: HISTORY_SCHEMA_VERSION,
			id: 9,
			started_at: "2026-04-12T00:00:00Z".to_string(),
			finished_at: "2026-04-12T00:01:00Z".to_string(),
			status: HistoryStatus::Applied,
			requested_by: "user (1000)".to_string(),
			command: "remove b".to_string(),
			requested_targets: vec!["b".to_string()],
			packages: vec![],
		},
	];

	assert_eq!(HistoryEntry::find(&entries, 9).unwrap().command, "remove b");
	assert!(HistoryEntry::find(&entries, 2).is_err());
}

#[test]
fn history_entry_selects_last_by_max_recorded_id() {
	let entries = vec![
		HistoryEntry {
			schema_version: HISTORY_SCHEMA_VERSION,
			id: 4,
			started_at: "2026-04-11T00:00:00Z".to_string(),
			finished_at: "2026-04-11T00:01:00Z".to_string(),
			status: HistoryStatus::Applied,
			requested_by: "user (1000)".to_string(),
			command: "install a".to_string(),
			requested_targets: vec!["a".to_string()],
			packages: vec![],
		},
		HistoryEntry {
			schema_version: HISTORY_SCHEMA_VERSION,
			id: 9,
			started_at: "2026-04-12T00:00:00Z".to_string(),
			finished_at: "2026-04-12T00:01:00Z".to_string(),
			status: HistoryStatus::Applied,
			requested_by: "user (1000)".to_string(),
			command: "remove b".to_string(),
			requested_targets: vec!["b".to_string()],
			packages: vec![],
		},
	];

	assert_eq!(
		HistoryEntry::find_selector(&entries, &HistorySelector::Last)
			.unwrap()
			.id,
		9
	);
}

#[test]
fn format_history_timestamp_falls_back_to_original_value() {
	assert_eq!(
		HistoryEntry::format_timestamp("not-a-timestamp"),
		"not-a-timestamp"
	);
}

#[test]
fn history_package_set_groups_packages_by_operation() {
	let entry = HistoryEntry {
		schema_version: HISTORY_SCHEMA_VERSION,
		id: 1,
		started_at: "2026-04-11T00:00:00Z".to_string(),
		finished_at: "2026-04-11T00:01:00Z".to_string(),
		status: HistoryStatus::Applied,
		requested_by: "user (1000)".to_string(),
		command: "upgrade".to_string(),
		requested_targets: vec![],
		packages: vec![
			PackageTransition::transition(
				"demo".to_string(),
				1,
				Operation::Install,
				PackageState::missing(),
				PackageState::config_only(Some("1.0".to_string()), Some(false)),
			),
			PackageTransition::transition(
				"demo-old".to_string(),
				1,
				Operation::Remove,
				PackageState::config_only(Some("0.9".to_string()), Some(true)),
				PackageState::missing(),
			),
		],
	};

	let pkg_set = entry.grouped_packages();

	assert_eq!(pkg_set.get(&Operation::Install).unwrap().len(), 1);
	assert_eq!(pkg_set.get(&Operation::Remove).unwrap().len(), 1);
}

#[test]
fn undo_action_purges_new_install_when_package_was_missing() {
	let pkg = PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::Install,
		PackageState::missing(),
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
	);

	assert_eq!(pkg.undo_action().unwrap(), ReplayAction::Remove { purge: true });
}

#[test]
fn undo_action_removes_without_purge_when_install_restored_config_files() {
	let pkg = PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::Install,
		PackageState::config_only(Some("1.0".to_string()), Some(false)),
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
	);

	assert_eq!(pkg.undo_action().unwrap(), ReplayAction::Remove { purge: false });
}

#[test]
fn undo_action_restores_version_and_auto_state_for_removed_package() {
	let pkg = PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::AutoRemove,
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(true),
			config_files_only: false,
		},
		PackageState::config_only(Some("1.0".to_string()), Some(true)),
	);

	assert_eq!(
		pkg.undo_action().unwrap(),
		ReplayAction::Install {
			version: "1.0".to_string(),
			auto_installed: Some(true),
		}
	);
}

#[test]
fn undo_action_rejects_reinstall_entries() {
	let pkg = PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::Reinstall,
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
	);

	assert!(pkg.undo_action().is_err());
}

#[test]
fn undo_action_rejects_purge_of_config_only_state() {
	let pkg = PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::Purge,
		PackageState::config_only(Some("1.0".to_string()), Some(false)),
		PackageState::missing(),
	);

	assert!(pkg.undo_action().is_err());
}

#[test]
fn redo_action_replays_target_version_and_auto_state() {
	let pkg = PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::Upgrade,
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(true),
			config_files_only: false,
		},
		PackageState {
			version: Some("2.0".to_string()),
			auto_installed: Some(true),
			config_files_only: false,
		},
	);

	assert_eq!(
		pkg.redo_action().unwrap(),
		ReplayAction::Install {
			version: "2.0".to_string(),
			auto_installed: Some(true),
		}
	);
}

#[test]
fn redo_action_preserves_remove_vs_purge() {
	let remove = PackageTransition::transition(
		"demo-remove".to_string(),
		1,
		Operation::Remove,
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
		PackageState::config_only(Some("1.0".to_string()), Some(false)),
	);
	let purge = PackageTransition::transition(
		"demo-purge".to_string(),
		1,
		Operation::Purge,
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
		PackageState::missing(),
	);

	assert_eq!(
		remove.redo_action().unwrap(),
		ReplayAction::Remove { purge: false }
	);
	assert_eq!(
		purge.redo_action().unwrap(),
		ReplayAction::Remove { purge: true }
	);
}

#[test]
fn redo_action_replays_reinstall_entries() {
	let pkg = PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::Reinstall,
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
		PackageState {
			version: Some("1.0".to_string()),
			auto_installed: Some(false),
			config_files_only: false,
		},
	);

	assert_eq!(
		pkg.redo_action().unwrap(),
		ReplayAction::Reinstall {
			version: "1.0".to_string(),
			auto_installed: Some(false),
		}
	);
}

#[test]
fn history_entry_json_roundtrip_preserves_recorded_fields() {
	let entry = HistoryEntry {
		schema_version: HISTORY_SCHEMA_VERSION,
		id: 17,
		started_at: "2026-04-11T00:00:00Z".to_string(),
		finished_at: "2026-04-11T00:01:00Z".to_string(),
		status: HistoryStatus::Applied,
		requested_by: "user (1000)".to_string(),
		command: "install demo".to_string(),
		requested_targets: vec!["demo".to_string()],
		packages: vec![PackageTransition::transition(
			"demo".to_string(),
			1,
			Operation::Install,
			PackageState::missing(),
			PackageState {
				version: Some("1.0".to_string()),
				auto_installed: Some(false),
				config_files_only: false,
			},
		)],
	};

	let json = serde_json::to_string_pretty(&entry).unwrap();
	assert!(json.contains("\"schema_version\": 1"));
	assert!(json.contains("\"command\": \"install demo\""));

	let decoded: HistoryEntry = serde_json::from_str(&json).unwrap();
	assert_eq!(decoded.id, 17);
	assert_eq!(decoded.status, HistoryStatus::Applied);
	assert_eq!(decoded.packages.len(), 1);
	assert_eq!(decoded.packages[0].name, "demo");
}

fn temp_history_dir() -> PathBuf {
	let unique = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_nanos();
	std::env::temp_dir().join(format!("nala-history-test-{unique}"))
}

fn sample_entry(id: u32, command: &str) -> HistoryEntry {
	HistoryEntry {
		schema_version: HISTORY_SCHEMA_VERSION,
		id,
		started_at: "2026-04-11T00:00:00Z".to_string(),
		finished_at: "2026-04-11T00:01:00Z".to_string(),
		status: HistoryStatus::Applied,
		requested_by: "user (1000)".to_string(),
		command: command.to_string(),
		requested_targets: vec![],
		packages: vec![],
	}
}

#[test]
fn clear_history_removes_selected_entry_only() {
	let _guard = history_store_test_lock();
	let runtime = Runtime::new().unwrap();
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	let first = sample_entry(3, "install demo");
	let second = sample_entry(8, "remove demo");
	first.write_to_file(&config).unwrap();
	second.write_to_file(&config).unwrap();

	let entries = runtime.block_on(get_history(&config)).unwrap();
	let removed = runtime
		.block_on(clear_history(
			&config,
			&entries,
			Some(&HistorySelector::Id(3)),
			false,
		))
		.unwrap();

	assert_eq!(removed, 1);
	assert!(!history_dir.join("3.json").exists());
	assert!(history_dir.join("8.json").exists());

	let remaining = runtime.block_on(get_history(&config)).unwrap();
	assert_eq!(remaining.len(), 1);
	assert_eq!(remaining[0].id, 8);

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn clear_history_supports_last_selector() {
	let _guard = history_store_test_lock();
	let runtime = Runtime::new().unwrap();
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	sample_entry(2, "install a").write_to_file(&config).unwrap();
	sample_entry(9, "install b").write_to_file(&config).unwrap();

	let entries = runtime.block_on(get_history(&config)).unwrap();
	runtime
		.block_on(clear_history(
			&config,
			&entries,
			Some(&HistorySelector::Last),
			false,
		))
		.unwrap();

	assert!(history_dir.join("2.json").exists());
	assert!(!history_dir.join("9.json").exists());

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn clear_history_all_removes_every_stored_entry() {
	let _guard = history_store_test_lock();
	let runtime = Runtime::new().unwrap();
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	sample_entry(1, "install a").write_to_file(&config).unwrap();
	sample_entry(2, "remove b").write_to_file(&config).unwrap();
	fs::write(history_dir.join("3.json"), "{").unwrap();
	fs::write(history_dir.join("1.json.bak"), "{}").unwrap();

	let removed = runtime
		.block_on(clear_history(&config, &[], None, true))
		.unwrap();

	assert_eq!(removed, 3);
	assert!(!history_dir.join("1.json").exists());
	assert!(!history_dir.join("2.json").exists());
	assert!(!history_dir.join("3.json").exists());
	assert!(history_dir.join("1.json.bak").exists());
	assert!(runtime.block_on(get_history(&config)).unwrap().is_empty());

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn get_history_ignores_non_history_files() {
	let _guard = history_store_test_lock();
	let runtime = Runtime::new().unwrap();
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	sample_entry(4, "install a").write_to_file(&config).unwrap();
	fs::write(history_dir.join("1.json.bak"), "{").unwrap();
	fs::write(history_dir.join("notes.txt"), "{").unwrap();

	let entries = runtime.block_on(get_history(&config)).unwrap();

	assert_eq!(entries.len(), 1);
	assert_eq!(entries[0].id, 4);

	fs::remove_dir_all(&history_dir).unwrap();
}
