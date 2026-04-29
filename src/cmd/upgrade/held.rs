use std::collections::HashSet;

use rust_apt::{Cache, Package, PackageSort, PkgSelectedState};

use crate::libnala::{
	package_key, HeldReason, PackageKey, PackageState, PackageTransition,
};

#[derive(Debug, Clone)]
pub(super) struct UpgradeSnapshot {
	key: PackageKey,
	name: String,
	size: u64,
	before: PackageState,
	after: PackageState,
	manual_hold: bool,
	phasing_applied: bool,
	phased_percentage: Option<u8>,
}

impl UpgradeSnapshot {
	fn from_package(pkg: &Package<'_>) -> Option<Self> {
		let (Some(installed), Some(candidate)) = (pkg.installed(), pkg.candidate()) else {
			return None;
		};

		Some(Self {
			key: package_key(pkg),
			name: pkg.name().to_string(),
			size: candidate.size(),
			before: PackageState::from_version(&installed),
			after: PackageState::from_version(&candidate),
			manual_hold: pkg.selected_state() == PkgSelectedState::Hold,
			phasing_applied: pkg.phasing_applied(),
			phased_percentage: candidate.phased_update_percentage(),
		})
	}

	fn held_reason(&self, protected: &HashSet<PackageKey>) -> HeldReason {
		if protected.contains(&self.key) {
			return HeldReason::Excluded;
		}

		if self.manual_hold {
			return HeldReason::ManualHold;
		}

		if self.phasing_applied {
			return HeldReason::PhasedUpdate {
				percentage: self.phased_percentage,
			};
		}

		HeldReason::KeptBack
	}

	fn held_transition(&self, protected: &HashSet<PackageKey>) -> PackageTransition {
		PackageTransition::held(
			self.name.clone(),
			self.size,
			self.before.clone(),
			self.after.clone(),
			self.held_reason(protected),
		)
	}
}

pub(super) fn snapshots(cache: &Cache) -> Vec<UpgradeSnapshot> {
	let sort = PackageSort::default().upgradable().names();
	cache
		.packages(&sort)
		.filter_map(|pkg| UpgradeSnapshot::from_package(&pkg))
		.collect()
}

pub(super) fn transitions(
	snapshots: Vec<UpgradeSnapshot>,
	changed: &HashSet<PackageKey>,
	protected: &HashSet<PackageKey>,
) -> Vec<PackageTransition> {
	snapshots
		.into_iter()
		.filter(|snapshot| !changed.contains(&snapshot.key))
		.map(|snapshot| snapshot.held_transition(protected))
		.collect()
}
