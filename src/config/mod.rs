pub mod color;
pub mod configuration;
pub mod logger;
pub mod paths;

pub use color::Theme;
pub use configuration::Config;
pub use logger::{setup_logger, Level};
pub use paths::Paths;
use serde::{Deserialize, Serialize};

use crate::tui::UnitStr;

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum Switch {
	Always,
	Never,
	Auto,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum OptType {
	Bool(bool),
	Int(u8),
	Int64(u64),
	Switch(Switch),
	UnitStr(UnitStr),
	// Strings have to be last in the enum
	// as almost anything will match them
	String(String),
	VecString(Vec<String>),
}
