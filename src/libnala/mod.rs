mod transaction;

use std::collections::{HashMap, HashSet};

use anyhow::{bail, Result};
use rust_apt::{Cache, Marked, Package, PkgCurrentState};
pub use transaction::{Operation, PackageState, PackageTransition};

use crate::config::color;
use crate::{debug, info, warn};

type SortedChanges<'a> = (Vec<Package<'a>>, HashMap<Operation, Vec<PackageTransition>>);

fn current_package_state(pkg: &Package<'_>) -> Option<PackageState> {
	if let Some(installed) = pkg.installed() {
		return Some(PackageState::from_version(&installed));
	}

	if pkg.config_state() {
		return Some(PackageState::config_only(
			None,
			Some(pkg.is_auto_installed()),
		));
	}

	None
}

// Package is not really mutable in the way clippy thinks.
#[allow(clippy::mutable_key_type)]
pub trait NalaCache {
	fn sort_changes<'a>(&'a self, auto: HashSet<Package<'a>>) -> Result<SortedChanges<'a>>;
	fn auto_remove(&self, remove_config: bool, purge: bool) -> HashSet<Package<'_>>;
}

pub trait PackageExt<'a> {
	fn filter_virtual(self) -> Result<Package<'a>>;
	fn config_state(&self) -> bool;
}

impl<'a> PackageExt<'a> for Package<'a> {
	fn filter_virtual(self) -> Result<Package<'a>> {
		if self.has_versions() {
			return Ok(self);
		}

		// Package is virtual so get its providers.
		// HashSet for duplicated packages when there is more than one version
		// clippy thinks that the package is mutable
		// But it only hashes the ID and you can't really mutate a package
		#[allow(clippy::mutable_key_type)]
		let providers: HashSet<Package> = self.provides().map(|p| p.package()).collect();

		// If the package doesn't have provides it's purely virtual
		// There is nothing that can satisfy it. Referenced only by name
		// At time of commit `python3-libmapper` is purely virtual
		if providers.is_empty() {
			warn!(
				"{} has no providers and is purely virutal",
				color::primary!(self.name())
			);

			return Ok(self);
		}

		// If there is only one provider just select that as the target
		if providers.len() == 1 {
			// Unwrap should be fine here, we know that there is 1 in the Vector.
			let target = providers.into_iter().next().unwrap();
			info!(
				"Selecting {} instead of virtual package {}",
				color::primary!(target.fullname(false)),
				color::primary!(self.name())
			);
			return Ok(target);
		}

		// If there are multiple providers then we will error out
		// and show the packages the user could select instead.
		info!(
			"{} is a virtual package provided by:",
			color::primary!(self.name())
		);

		for target in &providers {
			// If the version doesn't have a candidate no sense in showing it
			if let Some(cand) = target.candidate() {
				println!(
					"    {} {}",
					color::primary!(target.fullname(true)),
					color::ver!(cand.version()),
				);
			}
		}
		bail!("You should select just one.")
	}

	fn config_state(&self) -> bool { self.current_state() == PkgCurrentState::ConfigFiles }
}

impl NalaCache for Cache {
	/// Run the autoremover and then get the changes from the cache.
	fn sort_changes<'a>(&'a self, auto: HashSet<Package<'a>>) -> Result<SortedChanges<'a>> {
		let mut pkg_set: HashMap<Operation, Vec<PackageTransition>> = HashMap::new();
		let mut pkgs: Vec<Package> = vec![];

		debug!("Calculating changes");
		let changed = self.get_changes(true).collect::<Vec<_>>();
		if changed.is_empty() {
			return Ok((vec![], pkg_set));
		}

		for pkg in changed {
			debug!("{pkg}:");
			debug!("  Marked::{:?}", pkg.marked());

			match pkg.marked() {
				mark @ (Marked::NewInstall | Marked::Install | Marked::ReInstall) => {
					let Some(after_version) = pkg.install_version() else {
						continue;
					};
					let op = match mark {
						Marked::ReInstall => Operation::Reinstall,
						_ => Operation::Install,
					};
					let before = current_package_state(&pkg).unwrap_or_else(PackageState::missing);
					let after = PackageState::from_version(&after_version);

					debug!("  Operation::{op:?}");
					pkg_set
						.entry(op)
						.or_default()
						.push(PackageTransition::transition(
							pkg.name().to_string(),
							after_version.size(),
							op,
							before,
							after,
						));
					pkgs.push(pkg);
				},
				mark @ (Marked::Remove | Marked::Purge) => {
					let Some(before) = current_package_state(&pkg) else {
						continue;
					};

					let op = if auto.contains(&pkg) {
						match mark {
							Marked::Remove => Operation::AutoRemove,
							Marked::Purge => Operation::AutoPurge,
							_ => unreachable!(),
						}
					} else {
						match mark {
							Marked::Remove => Operation::Remove,
							Marked::Purge => Operation::Purge,
							_ => unreachable!(),
						}
					};

					let after = match op {
						Operation::Remove | Operation::AutoRemove => {
							PackageState::config_only(before.version.clone(), before.auto_installed)
						},
						Operation::Purge | Operation::AutoPurge => PackageState::missing(),
						_ => unreachable!(),
					};

					let size = pkg
						.installed()
						.map(|installed| installed.size())
						.unwrap_or_default();

					debug!("  Operation::{op:?}");
					pkg_set
						.entry(op)
						.or_default()
						.push(PackageTransition::transition(
							pkg.name().to_string(),
							size,
							op,
							before,
							after,
						));
					pkgs.push(pkg);
				},
				mark @ (Marked::Upgrade | Marked::Downgrade) => {
					if let (Some(inst), Some(cand)) = (pkg.installed(), pkg.candidate()) {
						let op = match mark {
							Marked::Upgrade => Operation::Upgrade,
							_ => Operation::Downgrade,
						};

						debug!("  Operation::{op:?}");
						pkg_set
							.entry(op)
							.or_default()
							.push(PackageTransition::transition(
								pkg.name().to_string(),
								cand.size(),
								op,
								PackageState::from_version(&inst),
								PackageState::from_version(&cand),
							));

						pkgs.push(pkg)
					}
				},
				// TODO: See if pkg is held for phasing and show percent
				// pkgDepCache::PhasingApplied
				// VerIterator::PhasedUpdatePercentage
				Marked::Held => {
					let Some(cand) = pkg.candidate() else {
						continue;
					};
					let before = current_package_state(&pkg).unwrap_or_else(PackageState::missing);
					let op = Operation::Held;

					debug!("  Operation::{op:?}");
					pkg_set
						.entry(op)
						.or_default()
						.push(PackageTransition::transition(
							pkg.name().to_string(),
							cand.size(),
							op,
							before,
							PackageState::from_version(&cand),
						));
				},
				Marked::Keep => continue,
				Marked::None => bail!("{pkg} not marked, this should be impossible"),
			}
		}

		Ok((pkgs, pkg_set))
	}

	fn auto_remove(&self, remove_config: bool, purge: bool) -> HashSet<Package<'_>> {
		// Package is not really mutable in the way clippy thinks.
		#[allow(clippy::mutable_key_type)]
		let mut set = HashSet::new();
		debug!("Auto Remover:");
		let _ = unsafe { self.depcache().action_group() };
		for pkg in self.iter() {
			// TODO: Should we have --remove-config, or just do it like apt does and match
			// on state? apt purge ~c is the equivalent.
			if !pkg.is_installed() && pkg.config_state() && remove_config && purge {
				pkg.mark_delete(purge);
				set.insert(pkg);
				continue;
			}

			if !pkg.is_auto_removable() || pkg.marked_delete() {
				continue;
			}

			if !pkg.config_state() {
				pkg.mark_delete(purge);
				set.insert(pkg);
			} else {
				pkg.mark_keep();
			}
		}
		// There is more code in private-install.cc DoAutomaticremove
		// If there are auto_remove bugs consider implementing that.
		set
	}
}
