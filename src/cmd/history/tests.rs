use super::replay::ReplayAction;
use super::*;
use super::model::{HistoryStatus, HISTORY_SCHEMA_VERSION};
use crate::cli::HistorySelector;
use crate::config::Config;
use crate::libnala::{Operation, PackageState, PackageTransition};

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
			altered: 1,
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
			altered: 1,
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
			altered: 1,
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
			altered: 1,
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
			altered: 2,
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
fn redo_action_rejects_reinstall_entries() {
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

	assert!(pkg.redo_action().is_err());
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
			altered: 1,
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
	assert_eq!(decoded.packages().len(), 1);
	assert_eq!(decoded.packages()[0].name, "demo");
}
