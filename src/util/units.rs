use serde::{Deserialize, Serialize};

/// Numeral System for unit conversion.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy, Default)]
pub enum NumSys {
	/// Base 2 | 1024 | KibiByte (KiB)
	#[default]
	Binary,
	/// Base 10 | 1000 | KiloByte (KB)
	Decimal,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct UnitStr {
	#[serde(default)]
	precision: usize,
	base: NumSys,
}

impl Default for UnitStr {
	fn default() -> Self { Self::new(0, NumSys::Binary) }
}

impl UnitStr {
	pub fn new(precision: usize, base: NumSys) -> UnitStr { UnitStr { precision, base } }

	pub fn str(&self, val: u64) -> String {
		let val = val as f64;
		let (num, tera, giga, mega, kilo) = match self.base {
			NumSys::Binary => (1024.0_f64, "TiB", "GiB", "MiB", "KiB"),
			NumSys::Decimal => (1000.0_f64, "TB", "GB", "MB", "KB"),
		};

		let powers = [
			(num.powi(4), tera),
			(num.powi(3), giga),
			(num.powi(2), mega),
			(num, kilo),
		];

		for (divisor, unit) in powers {
			if val > divisor {
				return format!("{:.1$} {unit}", val / divisor, self.precision);
			}
		}
		format!("{val} B")
	}

	pub fn base(self) -> NumSys { self.base }
}
