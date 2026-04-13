use anyhow::{bail, Result};
use rust_apt::util::show_broken_pkg;
use rust_apt::{new_cache, Cache};

use super::model::{HistoryEntry, HistoryStatus};
use crate::config::Config;
use crate::libnala::PackageTransition;
use crate::{debug, util};

/// Solver-facing package action derived from a stored history transition.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ReplayAction {
	Install {
		version: String,
		auto_installed: Option<bool>,
	},
	Remove {
		purge: bool,
	},
}

impl HistoryEntry {
	/// Marks the inverse of this entry's package effects into the cache and commits them.
	pub async fn undo(&self, config: &mut Config) -> Result<()> {
		let cache = self.prepare_replay(config)?;

		for package in self.packages() {
			package.mark_undo(&cache)?;
		}

		Self::commit_replay(cache, config).await
	}

	/// Replays this entry's recorded package effects into the cache and commits them.
	pub async fn redo(&self, config: &mut Config) -> Result<()> {
		let cache = self.prepare_replay(config)?;

		for package in self.packages() {
			package.mark_redo(&cache)?;
		}

		Self::commit_replay(cache, config).await
	}

	fn prepare_replay(&self, config: &mut Config) -> Result<Cache> {
		if self.status != HistoryStatus::Applied {
			bail!(
				"History entry '{}' is not replayable because it was not recorded as applied",
				self.id
			);
		}

		if self.packages().is_empty() {
			bail!("History entry '{}' has no package changes to replay", self.id);
		}

		util::sudo_check(config)?;
		config.set_bool(crate::config::keys::NO_AUTO_REMOVE, true);

		Ok(new_cache!()?)
	}

	async fn commit_replay(cache: Cache, config: &mut Config) -> Result<()> {
		if let Err(err) = cache.resolve(false) {
			debug!("Broken Count: {}", cache.depcache().broken_count());
			for pkg in cache.iter() {
				if let Some(broken) = show_broken_pkg(&cache, &pkg, false) {
					eprintln!("{broken}");
				}
			}
			bail!(err);
		}

		crate::summary::commit(cache, config).await
	}
}

impl ReplayAction {
	/// Applies this solver action to the package referenced by the recorded history row.
	fn apply(self, package: &PackageTransition, cache: &Cache) -> Result<()> {
		let pkg = package.get_pkg(cache)?;

		cache.resolver().clear(&pkg);
		pkg.protect();

		match self {
			Self::Install {
				version,
				auto_installed,
			} => {
				let Some(ver) = pkg.get_version(&version) else {
					bail!("Version '{}' not found for '{}'", version, package.name)
				};

				ver.set_candidate();
				pkg.mark_install(true, true);

				if let Some(auto_installed) = auto_installed {
					pkg.mark_auto(auto_installed);
				}
			},
			Self::Remove { purge } => {
				pkg.mark_delete(purge);
			},
		}

		Ok(())
	}
}

impl PackageTransition {
	/// Computes the inverse package action needed to undo this transition.
	pub(super) fn undo_action(&self) -> Result<ReplayAction> {
		match self.operation {
			crate::cmd::Operation::Install => {
				if self.before.config_files_only {
					return Ok(ReplayAction::Remove { purge: false });
				}

				if self.before.is_missing() {
					return Ok(ReplayAction::Remove { purge: true });
				}

				let Some(version) = self.before.version.clone() else {
					bail!(
						"Undo is not supported for '{}' because the prior version was not recorded",
						self.name
					)
				};

				Ok(ReplayAction::Install {
					version,
					auto_installed: self.before.auto_installed,
				})
			},
			crate::cmd::Operation::Remove
			| crate::cmd::Operation::AutoRemove
			| crate::cmd::Operation::Purge
			| crate::cmd::Operation::AutoPurge
			| crate::cmd::Operation::Upgrade
			| crate::cmd::Operation::Downgrade => {
				if self.before.config_files_only {
					bail!(
						"Undo is not supported for '{}' because restoring config-files-only state is not implemented",
						self.name
					);
				}

				let Some(version) = self.before.version.clone() else {
					bail!(
						"Undo is not supported for '{}' because the prior installed version was not recorded",
						self.name
					)
				};

				Ok(ReplayAction::Install {
					version,
					auto_installed: self.before.auto_installed,
				})
			},
			crate::cmd::Operation::Reinstall => bail!(
				"Undo is not supported for '{}' because reinstall replay is not implemented",
				self.name
			),
			crate::cmd::Operation::Held => bail!("Held package '{}' cannot be undone", self.name),
		}
	}

	/// Computes the package action needed to redo this transition.
	pub(super) fn redo_action(&self) -> Result<ReplayAction> {
		match self.operation {
			crate::cmd::Operation::Install
			| crate::cmd::Operation::Upgrade
			| crate::cmd::Operation::Downgrade => {
				let Some(version) = self.after.version.clone() else {
					bail!(
						"Redo is not supported for '{}' because the resulting version was not recorded",
						self.name
					)
				};

				Ok(ReplayAction::Install {
					version,
					auto_installed: self.after.auto_installed,
				})
			},
			crate::cmd::Operation::Remove | crate::cmd::Operation::AutoRemove => {
				Ok(ReplayAction::Remove { purge: false })
			},
			crate::cmd::Operation::Purge | crate::cmd::Operation::AutoPurge => {
				Ok(ReplayAction::Remove { purge: true })
			},
			crate::cmd::Operation::Reinstall => bail!(
				"Redo is not supported for '{}' because reinstall replay is not implemented",
				self.name
			),
				crate::cmd::Operation::Held => bail!("Held package '{}' cannot be redone", self.name),
			}
		}

	/// Marks the inverse of this package transition into the current cache.
	fn mark_undo(&self, cache: &Cache) -> Result<()> {
		self.undo_action()?.apply(self, cache)
	}

	/// Marks this package transition into the current cache.
	fn mark_redo(&self, cache: &Cache) -> Result<()> {
		self.redo_action()?.apply(self, cache)
	}
}
