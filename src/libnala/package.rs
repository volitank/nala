use std::collections::HashSet;

use anyhow::{bail, Result};
use rust_apt::records::RecordField;
use rust_apt::{Package, PkgCurrentState, Version};

use crate::config::color;

pub trait NalaPkg<'a> {
	fn filter_virtual(self) -> Result<Package<'a>>;
	fn config_state(&self) -> bool;
	fn now_version(&self) -> Option<Version<'a>>;
}

pub trait NalaVersion {
	fn get_filename(&self) -> String;
}

impl NalaVersion for Version<'_> {
	/// Return the package name. Checks if epoch is needed.
	fn get_filename(&self) -> String {
		let filename = self
			.get_record(RecordField::Filename)
			.expect("Record does not contain a filename!")
			.split_terminator('/')
			.last()
			.expect("Filename is malformed!")
			.to_string();

		if let Some(index) = self.version().find(':') {
			let epoch = format!("_{}%3a", &self.version()[..index]);
			return filename.replacen('_', &epoch, 1);
		}
		filename
	}
}

impl<'a> NalaPkg<'a> for Package<'a> {
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
			crate::warning!(
				"{} has no providers and is purely virutal",
				color::primary!(self.name())
			);

			return Ok(self);
		}

		// If there is only one provider just select that as the target
		if providers.len() == 1 {
			// Unwrap should be fine here, we know that there is 1 in the Vector.
			let target = providers.into_iter().next().unwrap();
			crate::notice!(
				"Selecting {} instead of virtual package {}",
				color::primary!(target.fullname(false)),
				color::primary!(self.name())
			);
			return Ok(target);
		}

		// If there are multiple providers then we will error out
		// and show the packages the user could select instead.
		crate::notice!(
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

	fn now_version(&self) -> Option<Version<'a>> {
		for ver in self.versions() {
			for pkg_file in ver.package_files() {
				if let Some(archive) = pkg_file.archive() {
					if archive == "now" {
						return Some(ver);
					}
				}
			}
		}
		None
	}
}
