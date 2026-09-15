use super::model::{HistoryStatus, HISTORY_SCHEMA_VERSION};
use super::replay::ReplayAction;
use super::store::{cleanup_stale_history, next_history_id};
use super::*;
use crate::cli::HistorySelector;
use crate::config::Config;
use crate::libnala::{Operation, PackageState, PackageTransition};
use std::fs;
use std::path::PathBuf;

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
fn history_entry_selects_by_id_or_last() {
	let entries = vec![sample_entry(4, "install a"), sample_entry(9, "remove b")];

	assert_eq!(
		HistoryEntry::find_selector(&entries, &HistorySelector::Id(9))
			.unwrap()
			.command,
		"remove b"
	);
	assert!(HistoryEntry::find_selector(&entries, &HistorySelector::Id(2)).is_err());
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
fn configure_counts_as_altered_but_not_replayable() {
	let mut entry = sample_entry(1, "install demo");
	let state = PackageState {
		version: Some("1.0".to_string()),
		auto_installed: Some(false),
		config_files_only: false,
	};
	entry.packages.push(PackageTransition::transition(
		"demo".to_string(),
		1,
		Operation::Configure,
		state.clone(),
		state,
	));

	assert_eq!(entry.altered().count(), 1);
	assert_eq!(entry.replayable().count(), 0);
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

	assert_eq!(
		pkg.undo_action().unwrap(),
		ReplayAction::Remove { purge: true }
	);
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

	assert_eq!(
		pkg.undo_action().unwrap(),
		ReplayAction::Remove { purge: false }
	);
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

#[test]
fn history_entry_accepts_beta_altered_field() {
	let mut value = serde_json::to_value(sample_entry(1, "upgrade")).unwrap();
	value["altered"] = serde_json::json!(18);

	assert_eq!(serde_json::from_value::<HistoryEntry>(value).unwrap().id, 1);
}

fn temp_history_dir() -> PathBuf {
	let template = std::env::temp_dir().join("nala-history-test-XXXXXX");
	nix::unistd::mkdtemp(&template).unwrap()
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
fn legacy_history_converts_to_current_entries() {
	let history_dir = temp_history_dir();
	let legacy_path = super::legacy::legacy_history_path(&history_dir);
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());
	sample_entry(1, "rust one").write_to_file(&config).unwrap();
	sample_entry(2, "rust two").write_to_file(&config).unwrap();

	let legacy = serde_json::json!({
		"1": {
			"Date": "2022-04-11 10:00:00 UTC",
			"Requested-By": "user (1000)",
			"Command": ["upgrade"],
			"Explicit": ["installed"],
			"Installed": [["installed", "1.0", "10"]],
			"Upgraded": [
				["modern-upgrade", "2.0", "11", "1.0"],
				["numeric-modern-upgrade", "12", "2", "11"],
				["numeric-old-modern-upgrade", "37~deb12u1", "5616", "35"],
				["date-version-modern-upgrade", "20230311+deb12u1", "155260", "20230311"],
				["old-upgrade", "1.0", "2.0", "12"],
				["missing-old-version", "2.0", "13"]
			],
			"Downgraded": [["old-downgrade", "20.0", "10.0", "12"]]
		},
		"Nala": {
			"History-Version": "1",
			"User-Installed": ["ignored"]
		}
	});
	let original = serde_json::to_vec_pretty(&legacy).unwrap();
	let mut invalid = legacy.clone();
	invalid["1"]["Installed"] = serde_json::json!([["invalid-install", "1", "2", "3"]]);
	let invalid = serde_json::to_vec_pretty(&invalid).unwrap();
	fs::write(&legacy_path, &invalid).unwrap();
	assert!(sample_entry(3, "must not write").write_to_file(&config).is_err());
	assert!(!history_dir.join("3.json").exists());
	assert_eq!(fs::read(&legacy_path).unwrap(), invalid);

	let mut invalid = legacy.clone();
	invalid["1"]["Upgraded"] = serde_json::json!([["invalid", "old", "new", "size"]]);
	let invalid = serde_json::to_vec_pretty(&invalid).unwrap();
	fs::write(&legacy_path, &invalid).unwrap();
	assert!(sample_entry(3, "must not write").write_to_file(&config).is_err());
	assert!(!history_dir.join("3.json").exists());
	assert_eq!(fs::read(&legacy_path).unwrap(), invalid);

	fs::write(&legacy_path, &original).unwrap();

	assert_eq!(next_history_id(&config).unwrap(), 4);
	sample_entry(4, "install current")
		.write_to_file(&config)
		.unwrap();
	let entries = get_history(&config).unwrap();

	assert_eq!(entries.len(), 4);
	assert_eq!(entries.iter().map(|entry| entry.id).collect::<Vec<_>>(), vec![1, 2, 3, 4]);
	assert_eq!(entries[1].command, "rust one");
	assert_eq!(entries[2].command, "rust two");
	assert_eq!(entries[3].command, "install current");
	assert_eq!(entries[0].status, HistoryStatus::Applied);
	assert_eq!(entries[0].requested_targets, vec!["installed"]);
	assert_eq!(
		entries[0]
			.packages
			.iter()
			.map(|package| (
				package.name.as_str(),
				package.before.version.as_deref(),
				package.before.config_files_only,
				package.after.version.as_deref(),
			))
			.collect::<Vec<_>>(),
		vec![
			("installed", None, true, Some("1.0")),
			("modern-upgrade", Some("1.0"), false, Some("2.0")),
			("numeric-modern-upgrade", Some("11"), false, Some("12")),
			(
				"numeric-old-modern-upgrade",
				Some("35"),
				false,
				Some("37~deb12u1"),
			),
			(
				"date-version-modern-upgrade",
				Some("20230311"),
				false,
				Some("20230311+deb12u1"),
			),
			("old-upgrade", Some("1.0"), false, Some("2.0")),
			("missing-old-version", None, false, Some("2.0")),
			("old-downgrade", Some("20.0"), false, Some("10.0")),
		]
	);
	assert!(entries[0].packages.iter().all(|package| {
		package.before.auto_installed.is_none() && package.after.auto_installed.is_none()
	}));

	assert!(history_dir.join("1.json").exists());
	assert!(history_dir.join("2.json").exists());
	assert!(history_dir.join("3.json").exists());
	assert!(history_dir.join("4.json").exists());
	assert_eq!(fs::read(&legacy_path).unwrap(), original);

	fs::remove_dir_all(history_dir).unwrap();
	fs::remove_file(legacy_path).unwrap();
}

#[test]
fn clear_history_removes_selected_entry_only() {
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	let first = sample_entry(3, "install demo");
	let second = sample_entry(8, "remove demo");
	first.write_to_file(&config).unwrap();
	second.write_to_file(&config).unwrap();

	let entries = get_history(&config).unwrap();
	let removed = clear_history(
		&config,
		&entries,
		Some(&HistorySelector::Id(3)),
		false,
	)
	.unwrap();

	assert_eq!(removed, 1);
	assert!(!history_dir.join("3.json").exists());
	assert!(history_dir.join("8.json").exists());

	let remaining = get_history(&config).unwrap();
	assert_eq!(remaining.len(), 1);
	assert_eq!(remaining[0].id, 8);

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn clear_history_supports_last_selector() {
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	sample_entry(2, "install a").write_to_file(&config).unwrap();
	sample_entry(9, "install b").write_to_file(&config).unwrap();

	let entries = get_history(&config).unwrap();
	clear_history(
		&config,
		&entries,
		Some(&HistorySelector::Last),
		false,
	)
	.unwrap();

	assert!(history_dir.join("2.json").exists());
	assert!(!history_dir.join("9.json").exists());

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn clear_history_all_removes_every_stored_entry() {
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	sample_entry(1, "install a").write_to_file(&config).unwrap();
	sample_entry(2, "remove b").write_to_file(&config).unwrap();
	fs::write(history_dir.join("3.json"), "{").unwrap();
	fs::write(history_dir.join("1.json.bak"), "{}").unwrap();
	assert!(get_history(&config).is_err());

	let removed = clear_history(&config, &[], None, true).unwrap();

	assert_eq!(removed, 3);
	assert!(!history_dir.join("1.json").exists());
	assert!(!history_dir.join("2.json").exists());
	assert!(!history_dir.join("3.json").exists());
	assert!(history_dir.join("1.json.bak").exists());
	assert!(get_history(&config).unwrap().is_empty());

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn get_history_ignores_non_history_files() {
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	sample_entry(4, "install a").write_to_file(&config).unwrap();
	fs::write(history_dir.join("1.json.bak"), "{").unwrap();
	fs::write(history_dir.join("5.json.tmp"), "{").unwrap();
	fs::write(history_dir.join("notes.txt"), "{").unwrap();

	let entries = get_history(&config).unwrap();

	assert_eq!(entries.len(), 1);
	assert_eq!(entries[0].id, 4);

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn get_history_validates_schema_version_and_filename_id() {
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());
	let path = history_dir.join("1.json");
	sample_entry(1, "install a").write_to_file(&config).unwrap();

	assert_eq!(get_history(&config).unwrap()[0].schema_version, 1);

	let mut value: serde_json::Value =
		serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
	value.as_object_mut().unwrap().remove("schema_version");
	fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
	assert!(get_history(&config)
		.unwrap_err()
		.to_string()
		.contains("no valid schema_version"));

	value["schema_version"] = serde_json::json!(2);
	fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
	let error = get_history(&config).unwrap_err().to_string();
	assert!(error.contains("unsupported schema version 2"));
	assert!(error.contains(path.to_str().unwrap()));

	value["schema_version"] = serde_json::json!(1);
	value["id"] = serde_json::json!(2);
	fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
	let error = get_history(&config).unwrap_err().to_string();
	assert!(error.contains("contains ID 2; expected 1"));

	fs::remove_dir_all(&history_dir).unwrap();
}

#[test]
fn stale_history_cleanup_only_removes_nala_temporary_state() {
	let history_dir = temp_history_dir();
	let mut config = Config::default();
	config.set_history_dir(history_dir.to_string_lossy());

	fs::write(history_dir.join("5.json.tmp"), "{").unwrap();
	fs::write(history_dir.join("notes.tmp"), "keep").unwrap();
	let staging = history_dir.with_file_name(format!(
		".{}.importing-123",
		history_dir.file_name().unwrap().to_string_lossy()
	));
	fs::create_dir(&staging).unwrap();
	fs::write(staging.join("1.json"), "{}").unwrap();

	cleanup_stale_history(&config).unwrap();

	assert!(!history_dir.join("5.json.tmp").exists());
	assert!(history_dir.join("notes.tmp").exists());
	assert!(!staging.exists());

	fs::remove_dir_all(&history_dir).unwrap();
}
