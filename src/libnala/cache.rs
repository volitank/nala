use std::collections::{HashMap, HashSet};

use anyhow::{bail, Result};
use rust_apt::{Cache, Marked, Package};

use super::{HistoryPackage, Operation};
use crate::libnala::NalaPkg;

type SortedChanges<'a> = (Vec<Package<'a>>, HashMap<Operation, Vec<HistoryPackage>>);

// Package is not really mutable in the way clippy thinks.
#[allow(clippy::mutable_key_type)]
pub trait NalaCache {
	fn sort_changes<'a>(&'a self, auto: HashSet<Package<'a>>) -> Result<SortedChanges<'a>>;
	fn auto_remove(&self, remove_config: bool, purge: bool) -> HashSet<Package<'_>>;
}

impl NalaCache for Cache {
	/// Run the autoremover and then get the changes from the cache.
	fn sort_changes<'a>(&'a self, auto: HashSet<Package<'a>>) -> Result<SortedChanges<'a>> {
		let mut pkg_set: HashMap<Operation, Vec<HistoryPackage>> = HashMap::new();
		let mut pkgs: Vec<Package> = vec![];

		crate::debug!("Calculating changes");
		let changed = self.get_changes(true).collect::<Vec<_>>();
		if changed.is_empty() {
			return Ok((vec![], pkg_set));
		}

		for pkg in changed {
			crate::debug!("{pkg}:");
			crate::debug!("  Marked::{:?}", pkg.marked());

			let (op, ver) = match pkg.marked() {
				mark @ (Marked::NewInstall | Marked::Install | Marked::ReInstall) => {
					let Some(cand) = pkg.install_version() else {
						continue;
					};
					let op = match mark {
						Marked::ReInstall => Operation::Reinstall,
						_ => Operation::Install,
					};
					(op, cand)
				},
				mark @ (Marked::Remove | Marked::Purge) => {
					let inst = if let Some(inst) = pkg.installed() {
						inst
					// If the pkg is in config_state and not installed
					// It can still be purged, but technically it's not
					// installed. TODO: For now just choose the first
					// version available. This can panic on real situations
					// so it needs to be fixed. For example if you remove a
					// package and it's config files stick around
					// And then for whatever reason that package is no longer
					// available from the cache this will panic when trying
					// to purge it. We need to be able to send no version
					// into the summary I guess.
					} else if pkg.config_state() {
						pkg.versions().next().unwrap()
					} else {
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
					(op, inst)
				},
				mark @ (Marked::Upgrade | Marked::Downgrade) => {
					if let (Some(inst), Some(cand)) = (pkg.installed(), pkg.candidate()) {
						let op = match mark {
							Marked::Upgrade => Operation::Upgrade,
							_ => Operation::Downgrade,
						};

						crate::debug!("  Operation::{op:?}");
						pkg_set
							.entry(op)
							.or_default()
							.push(HistoryPackage::from_version(op, &cand, &Some(inst)));

						pkgs.push(pkg)
					}
					continue;
				},
				// TODO: See if pkg is held for phasing and show percent
				// pkgDepCache::PhasingApplied
				// VerIterator::PhasedUpdatePercentage
				Marked::Held => {
					let Some(cand) = pkg.candidate() else {
						continue;
					};
					(Operation::Held, cand)
				},
				Marked::Keep => continue,
				Marked::None => bail!("{pkg} not marked, this should be impossible"),
			};

			crate::debug!("  Operation::{op:?}");
			pkg_set
				.entry(op)
				.or_default()
				.push(HistoryPackage::from_version(op, &ver, &None));

			pkgs.push(pkg);
		}

		Ok((pkgs, pkg_set))
	}

	fn auto_remove(&self, remove_config: bool, purge: bool) -> HashSet<Package<'_>> {
		// Package is not really mutable in the way clippy thinks.
		#[allow(clippy::mutable_key_type)]
		let mut set = HashSet::new();
		crate::debug!("Auto Remover:");
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
