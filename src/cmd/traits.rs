use indexmap::IndexSet;
use rust_apt::{BaseDep, DepType, Dependency, PackageFile, Provider, Version};

use crate::config::{color, Theme};

pub trait ShowFormat {
	fn format(&self) -> String;
}

impl ShowFormat for BaseDep<'_> {
	fn format(&self) -> String {
		// These Dependency types will be colored red
		let theme = if matches!(self.dep_type(), DepType::Conflicts | DepType::DpkgBreaks) {
			Theme::Error
		} else {
			Theme::Primary
		};

		if let Some(comp) = self.comp_type() {
			return format!(
				// libgnutls30 (>= 3.7.5)
				"{} {}{comp} {}{}",
				// There's a compare operator in the dependency.
				// Dang better have a version smh my head.
				color::color!(theme, self.target_package().name()),
				color::highlight!("("),
				color::color!(Theme::Secondary, self.version().unwrap()),
				color::highlight!(")"),
			);
		}
		color::color!(theme, self.target_package().name()).into()
	}
}

const DEP_BUFFER: &str = "\n    ";
const DEP_SEP: &str = " | ";
impl ShowFormat for &Vec<Dependency<'_>> {
	fn format(&self) -> String {
		let mut depends_string = String::new();
		// Get total deps number to include Or Dependencies
		let total_deps = self.len();

		// If there are more than 4 deps format with multiple lines
		if total_deps > 3 {
			depends_string += DEP_BUFFER;
		}

		let mut inner = IndexSet::new();
		for (i, dep) in self.iter().enumerate() {
			let target = dep.first().target_package().name();
			if inner.contains(target) {
				continue;
			}
			inner.insert(target);

			// Or Deps need to be formatted slightly different.
			if dep.is_or() {
				for (j, base_dep) in dep.iter().enumerate() {
					depends_string += &base_dep.format();
					if j + 1 != dep.len() {
						depends_string += DEP_SEP;
					}
				}
			} else {
				// Regular dependencies are more simple than Or
				depends_string += &dep.first().format();
			}

			depends_string += if total_deps > 3 {
				DEP_BUFFER
			// Only add the comma if it isn't the last.
			} else if i + 1 != total_deps {
				", "
			} else {
				" "
			};
		}
		depends_string.trim_end().to_string()
	}
}

impl ShowFormat for PackageFile<'_> {
	fn format(&self) -> String {
		let mut string = String::new();

		let Some(archive) = self.archive() else {
			return "ERROR:?".into();
		};

		if archive == "now" {
			return " [now]".into();
		}

		string += " [";
		for (key, postfix) in [
			(self.origin(), "/"),
			(self.codename(), " "),
			(self.component(), "] "),
		] {
			if let Some(value) = key {
				string += value;
			}
			string += postfix;
		}
		string
	}
}

impl ShowFormat for Vec<Provider<'_>> {
	fn format(&self) -> String {
		format!(
			"[{}]",
			self.iter()
				.map(|p| p.name())
				.collect::<Vec<&str>>()
				.join(", ")
		)
	}
}

impl ShowFormat for Version<'_> {
	fn format(&self) -> String {
		format!(
			"{} {}",
			color::primary!(&self.parent().fullname(true)),
			color::ver!(self.version()),
		)
	}
}
