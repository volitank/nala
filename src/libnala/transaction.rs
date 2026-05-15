use anyhow::{bail, Result};
use rust_apt::{Cache, Package, Version};
use serde::{Deserialize, Serialize};

use crate::config::Theme;

/// Transaction operation recorded and displayed by Nala.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Operation {
	Remove,
	AutoRemove,
	Purge,
	AutoPurge,
	Install,
	Reinstall,
	Upgrade,
	Downgrade,
	Configure,
	Held,
}

impl Operation {
	pub fn to_vec() -> Vec<Operation> {
		vec![
			Self::Remove,
			Self::AutoRemove,
			Self::Purge,
			Self::AutoPurge,
			Self::Install,
			Self::Reinstall,
			Self::Upgrade,
			Self::Downgrade,
			Self::Configure,
			Self::Held,
		]
	}

	pub fn as_str(&self) -> &str { self.as_ref() }

	pub fn is_replayable(&self) -> bool { !matches!(self, Self::Configure | Self::Held) }
}

impl std::fmt::Display for Operation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", AsRef::<str>::as_ref(self))
	}
}

impl AsRef<str> for Operation {
	fn as_ref(&self) -> &str {
		match self {
			Operation::Remove => "Remove",
			Operation::AutoRemove => "AutoRemove",
			Operation::Purge => "Purge",
			Operation::AutoPurge => "AutoPurge",
			Operation::Install => "Install",
			Operation::Reinstall => "ReInstall",
			Operation::Upgrade => "Upgrade",
			Operation::Downgrade => "Downgrade",
			Operation::Configure => "Configure",
			Operation::Held => "Held",
		}
	}
}

impl AsRef<Theme> for Operation {
	fn as_ref(&self) -> &Theme {
		match self {
			Self::Remove | Self::AutoRemove | Self::Purge | Self::AutoPurge => &Theme::Error,
			Self::Install | Self::Upgrade => &Theme::Secondary,
			Self::Reinstall | Self::Downgrade | Self::Configure | Self::Held => &Theme::Notice,
		}
	}
}

/// User-facing explanation for a package being kept out of an upgrade.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum HeldReason {
	Excluded,
	ManualHold,
	PhasedUpdate { percentage: Option<u8> },
	KeptBack,
}

impl HeldReason {
	pub fn summary(&self) -> String {
		match self {
			Self::Excluded => "Excluded".to_string(),
			Self::ManualHold => "Manual hold".to_string(),
			Self::PhasedUpdate {
				percentage: Some(percentage),
			} => format!("Phased {percentage}%"),
			Self::PhasedUpdate { percentage: None } => "Phased".to_string(),
			Self::KeptBack => "Kept back".to_string(),
		}
	}
}

/// Package state snapshot captured before or after a transaction effect.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackageState {
	pub version: Option<String>,
	pub auto_installed: Option<bool>,
	pub config_files_only: bool,
}

impl PackageState {
	/// Returns an empty state for packages that were absent at that side of the
	/// transition.
	pub fn missing() -> Self { Self::default() }

	/// Builds a state snapshot from a concrete APT version record.
	pub fn from_version(version: &Version) -> Self {
		Self {
			version: Some(version.version().to_string()),
			auto_installed: Some(version.parent().is_auto_installed()),
			config_files_only: false,
		}
	}

	/// Builds a state snapshot for packages that only have config files
	/// remaining.
	pub fn config_only(version: Option<String>, auto_installed: Option<bool>) -> Self {
		Self {
			version,
			auto_installed,
			config_files_only: true,
		}
	}

	/// Returns the recorded package version as a borrowed string when one
	/// exists.
	pub(crate) fn version_str(&self) -> Option<&str> { self.version.as_deref() }

	/// Returns true when the package was not present in any form for this state
	/// snapshot.
	pub(crate) fn is_missing(&self) -> bool { self.version.is_none() && !self.config_files_only }
}

/// Shared package transition record used by summary, history storage, and
/// replay.
#[derive(Serialize, Deserialize, Debug)]
pub struct PackageTransition {
	pub name: String,
	pub size: u64,
	pub operation: Operation,
	pub before: PackageState,
	pub after: PackageState,
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub held_reason: Option<HeldReason>,
}

impl PackageTransition {
	/// Constructs a package transition from explicit before/after state
	/// snapshots.
	pub fn transition(
		name: String,
		size: u64,
		operation: Operation,
		before: PackageState,
		after: PackageState,
	) -> PackageTransition {
		Self {
			name,
			size,
			operation,
			before,
			after,
			held_reason: None,
		}
	}

	/// Constructs a package transition with a held-back reason.
	pub fn held(
		name: String,
		size: u64,
		before: PackageState,
		after: PackageState,
		reason: HeldReason,
	) -> PackageTransition {
		Self {
			name,
			size,
			operation: Operation::Held,
			before,
			after,
			held_reason: Some(reason),
		}
	}

	/// Looks up the package referenced by this transition in the current cache.
	pub(crate) fn get_pkg<'a>(&self, cache: &'a Cache) -> Result<Package<'a>> {
		let Some(pkg) = cache.get(&self.name) else {
			bail!("Package '{}' not found in cache", self.name)
		};
		Ok(pkg)
	}

	/// Resolves the recorded version associated with this transition from the
	/// current cache.
	pub(crate) fn get_version<'a>(&self, cache: &'a Cache) -> Result<Version<'a>> {
		let Some(version) = self.after.version_str().or(self.before.version_str()) else {
			bail!("No recorded version for '{}'", self.name)
		};

		let Some(ver) = self.get_pkg(cache)?.get_version(version) else {
			bail!("Version '{}' not found for '{}'", version, self.name)
		};
		Ok(ver)
	}
}
