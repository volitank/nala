use anyhow::Result;

use crate::cli::{Commands, HistoryCommand};
use crate::config::{Config, keys};
use crate::util::{AptLockGuard, sudo_check};

#[derive(Debug, PartialEq, Eq)]
enum Access {
	ReadOnly,
	Write,
	Resolve,
}

pub(crate) struct CommandSetup {
	access: Access,
	pub update_early: bool,
}

impl CommandSetup {
	pub fn new(command: &Commands, config: &mut Config) -> Self {
		use Access::{ReadOnly, Resolve, Write};

		let requested = config.get_bool(keys::UPDATE, false);
		let automatic = requested || config.get_bool(keys::AUTO_UPDATE, true);
		let (access, update_early) = match command {
			Commands::FullUpgrade(_) => {
				config.set_bool(keys::FULL, true);
				(Resolve, automatic)
			},
			Commands::SafeUpgrade(_) => {
				config.set_bool(keys::SAFE, true);
				(Resolve, automatic)
			},
			Commands::Upgrade(_) => (Resolve, automatic),
			Commands::Purge(_) => {
				config.set_bool(keys::PURGE, true);
				(Resolve, requested)
			},
			Commands::Install(_) | Commands::Remove(_) => (Resolve, requested),
			Commands::AutoPurge(_) => {
				config.set_bool(keys::PURGE, true);
				(Write, requested)
			},
			Commands::AutoRemove(_) => (Write, requested),
			Commands::Clean(_) | Commands::Fetch(_) | Commands::Update(_) => (Write, false),
			Commands::History(args) => match args.command.as_ref() {
				Some(HistoryCommand::Undo(_) | HistoryCommand::Redo(_)) => (Resolve, false),
				Some(HistoryCommand::Clear(_)) => (Write, false),
				Some(HistoryCommand::Info(_)) | None => (ReadOnly, false),
			},
			Commands::List(_)
			| Commands::Search(_)
			| Commands::Show(_)
			| Commands::Policy(_)
			| Commands::Download(_)
			| Commands::Moo(_) => (ReadOnly, false),
		};
		Self {
			access,
			update_early: update_early && !config.get_bool(keys::NO_UPDATE, false),
		}
	}

	/// Return the guard to the caller so it spans update, planning and commit.
	pub fn acquire(&self, config: &Config) -> Result<Option<AptLockGuard>> {
		if self.access == Access::ReadOnly {
			return Ok(None);
		}
		sudo_check(config)?;
		let lock = AptLockGuard::acquire()?;
		if self.access == Access::Resolve {
			config.apply_solver_default()?;
		}
		Ok(Some(lock))
	}
}

#[cfg(test)]
mod tests {
	use clap::Parser;

	use super::*;
	use crate::cli::NalaParser;

	fn assert_command_default(command_name: &str, key: &str) {
		let command = NalaParser::try_parse_from(["nala", command_name])
			.unwrap()
			.command
			.unwrap();
		let mut config = Config::default();

		CommandSetup::new(&command, &mut config);

		assert!(
			config.get_bool(key, false),
			"{command_name} did not set {key}"
		);
	}

	#[test]
	fn semantic_commands_set_their_defaults() {
		assert_command_default("full-upgrade", keys::FULL);
		assert_command_default("dist-upgrade", keys::FULL);
		assert_command_default("safe-upgrade", keys::SAFE);
		assert_command_default("purge", keys::PURGE);
		assert_command_default("autopurge", keys::PURGE);
	}

	#[test]
	fn mutation_commands_are_the_only_commands_that_take_the_apt_lock() {
		for args in [
			&["nala", "clean"][..],
			&["nala", "fetch"][..],
			&["nala", "update"][..],
			&["nala", "upgrade"][..],
			&["nala", "full-upgrade"][..],
			&["nala", "safe-upgrade"][..],
			&["nala", "install", "demo"][..],
			&["nala", "remove", "demo"][..],
			&["nala", "purge", "demo"][..],
			&["nala", "autoremove"][..],
			&["nala", "autopurge"][..],
			&["nala", "history", "undo", "last"][..],
			&["nala", "history", "redo", "last"][..],
			&["nala", "history", "clear", "--all"][..],
		] {
			let command = NalaParser::try_parse_from(args).unwrap().command.unwrap();
			assert!(
				CommandSetup::new(&command, &mut Config::default()).access != Access::ReadOnly,
				"{args:?} did not take the APT lock"
			);
		}

		for args in [
			&["nala", "list"][..],
			&["nala", "search"][..],
			&["nala", "show"][..],
			&["nala", "policy"][..],
			&["nala", "download", "demo"][..],
			&["nala", "history"][..],
			&["nala", "history", "info", "last"][..],
			&["nala", "moo"][..],
		] {
			let command = NalaParser::try_parse_from(args).unwrap().command.unwrap();
			assert!(
				CommandSetup::new(&command, &mut Config::default()).access == Access::ReadOnly,
				"{args:?} unexpectedly took the APT lock"
			);
		}
	}
}
