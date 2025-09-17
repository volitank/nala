use serde::{Deserialize, Serialize};

use crate::config::Theme;

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
		]
	}
}

impl Operation {
	pub fn as_str(&self) -> &str { self.as_ref() }

	pub fn undo(&self) -> Operation {
		match self {
			Operation::Remove | Operation::Purge | Operation::AutoRemove | Operation::AutoPurge => {
				Operation::Install
			},
			Operation::Install => Operation::Remove,
			Operation::Reinstall => Operation::Reinstall,
			Operation::Upgrade => Operation::Downgrade,
			Operation::Downgrade => Operation::Upgrade,
			Operation::Held => Operation::Held,
		}
	}
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
			Operation::Held => "Held",
		}
	}
}

impl AsRef<Theme> for Operation {
	fn as_ref(&self) -> &Theme {
		match self {
			Self::Remove | Self::AutoRemove | Self::Purge | Self::AutoPurge => &Theme::Error,
			Self::Install | Self::Upgrade => &Theme::Secondary,
			Self::Reinstall | Self::Downgrade | Self::Held => &Theme::Notice,
		}
	}
}
