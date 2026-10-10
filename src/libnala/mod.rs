mod transaction;

use std::collections::{HashMap, HashSet};

use anyhow::{Result, bail};
use rust_apt::{Cache, Marked, Package, PkgCurrentState};
pub use transaction::{HeldReason, Operation, PackageState, PackageTransition};

use crate::config::color;
use crate::{debug, info, t, warn};

type SortedChanges<'a> = (Vec<Package<'a>>, HashMap<Operation, Vec<PackageTransition>>);
pub type PackageKey = (String, String);

pub fn package_key(pkg: &Package<'_>) -> PackageKey {
	(pkg.name().to_string(), pkg.arch().to_string())
}

pub(crate) fn needs_configure(pkg: &Package<'_>) -> bool {
	matches!(
		pkg.current_state(),
		PkgCurrentState::UnPacked
			| PkgCurrentState::HalfConfigured
			| PkgCurrentState::HalfInstalled
			| PkgCurrentState::TriggersAwaited
			| PkgCurrentState::TriggersPending
	)
}

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
	fn auto_remove(
		&self,
		remove_config: bool,
		purge: bool,
		protected: &HashSet<PackageKey>,
	) -> HashSet<Package<'_>>;
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
				"{}",
				t!(
					"pkg-virtual-no-providers",
					"package" => color::primary!(self.name())
				)
			);

			return Ok(self);
		}

		// If there is only one provider just select that as the target
		if providers.len() == 1 {
			// Unwrap should be fine here, we know that there is 1 in the
			// Vector.
			let target = providers.into_iter().next().unwrap();
			info!(
				"{}",
				t!(
					"pkg-virtual-select",
					"provider" => color::primary!(target.fullname(false)),
					"package" => color::primary!(self.name())
				)
			);
			return Ok(target);
		}

		// If there are multiple providers then we will error out
		// and show the packages the user could select instead.
		info!(
			"{}",
			t!(
				"pkg-virtual-providers",
				"package" => color::primary!(self.name())
			)
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
		bail!("{}", t!("pkg-virtual-one"))
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
		let changed_keys = changed.iter().map(package_key).collect::<HashSet<_>>();

		for pkg in changed {
			debug!("{pkg}:");
			let marked = if pkg.marked_reinstall() { Marked::ReInstall } else { pkg.marked() };
			debug!("  Marked::{marked:?}");

			match marked {
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
							pkg.fullname(true),
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
							pkg.fullname(true),
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
								pkg.fullname(true),
								cand.size(),
								op,
								PackageState::from_version(&inst),
								PackageState::from_version(&cand),
							));

						pkgs.push(pkg)
					}
				},
				Marked::Held | Marked::Keep => continue,
				Marked::None => bail!("{pkg} not marked, this should be impossible"),
			}
		}

		for pkg in self.iter() {
			if changed_keys.contains(&package_key(&pkg)) || !needs_configure(&pkg) {
				continue;
			}

			let Some(installed) = pkg.installed() else {
				continue;
			};

			debug!("  Operation::{:?}", Operation::Configure);
			pkg_set
				.entry(Operation::Configure)
				.or_default()
				.push(PackageTransition::transition(
					pkg.fullname(true),
					installed.size(),
					Operation::Configure,
					PackageState::from_version(&installed),
					PackageState::from_version(&installed),
				));
			pkgs.push(pkg);
		}

		Ok((pkgs, pkg_set))
	}

	fn auto_remove(
		&self,
		remove_config: bool,
		purge: bool,
		protected: &HashSet<PackageKey>,
	) -> HashSet<Package<'_>> {
		// Package is not really mutable in the way clippy thinks.
		#[allow(clippy::mutable_key_type)]
		let mut set = HashSet::new();
		debug!("Auto Remover:");
		let _action_group = unsafe { self.depcache().action_group() };
		for pkg in self.iter() {
			if protected.contains(&package_key(&pkg)) {
				pkg.mark_keep();
				continue;
			}

			// TODO: Should we have --remove-config, or just do it like apt does
			// and match on state? apt purge ~c is the equivalent.
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

#[cfg(test)]
mod tests {
	use std::fs;
	use std::path::PathBuf;

	use rust_apt::config::Config;

	use super::*;

	#[test]
	fn sort_changes_classifies_reinstall_before_keep() {
		struct Fixture {
			dir: PathBuf,
			previous: Vec<(&'static str, Option<String>)>,
		}

		impl Drop for Fixture {
			fn drop(&mut self) {
				let config = Config::new();
				for (key, value) in &self.previous {
					if let Some(value) = value {
						config.set(key, value);
					} else {
						config.clear(key);
					}
				}
				fs::remove_dir_all(&self.dir).ok();
			}
		}

		// Rebuilders may have no APT indexes or downloadable installed
		// packages.
		let dir =
			nix::unistd::mkdtemp(&std::env::temp_dir().join("nala-reinstall-test-XXXXXX")).unwrap();
		let config = Config::new();
		let settings = [
			("Dir::Etc::sourcelist", "-".to_string()),
			("Dir::Etc::sourceparts", "-".to_string()),
			("Dir::Etc::preferences", "-".to_string()),
			("Dir::Etc::preferencesparts", "-".to_string()),
			(
				"Dir::State::status",
				dir.join("status").to_str().unwrap().to_string(),
			),
			(
				"Dir::State::lists",
				dir.join("lists").to_str().unwrap().to_string(),
			),
			(
				"Dir::State::extended_states",
				dir.join("extended_states").to_str().unwrap().to_string(),
			),
			("Dir::Cache::pkgcache", String::new()),
			("Dir::Cache::srcpkgcache", String::new()),
		];
		let fixture = Fixture {
			dir,
			previous: settings
				.iter()
				.map(|(key, _)| (*key, config.contains(key).then(|| config.find(key, ""))))
				.collect(),
		};
		fs::create_dir(fixture.dir.join("lists")).unwrap();
		let package = "Package: nala-reinstall-test\nVersion: 1.0\nArchitecture: all\nMaintainer: \
		               Nala tests <blake@volian.org>\nInstalled-Size: 1\nDescription: Reinstall \
		               test fixture\n";
		fs::write(
			fixture.dir.join("status"),
			format!("{package}Status: install ok installed\n\n"),
		)
		.unwrap();
		let packages = fixture.dir.join("Packages");
		fs::write(
			&packages,
			format!("{package}Filename: nala-reinstall-test_1.0_all.deb\nSize: 1\n\n"),
		)
		.unwrap();
		for (key, value) in &settings {
			config.set(key, value);
		}
		let cache = rust_apt::new_cache!(&[packages.to_str().unwrap()]).unwrap();
		assert_eq!(cache.iter().count(), 1);
		let pkg = cache.get("nala-reinstall-test").unwrap();

		cache.resolver().clear(&pkg);
		cache.resolver().protect(&pkg);
		pkg.mark_reinstall(true);
		cache.resolve(false).unwrap();
		assert!(pkg.marked_reinstall());
		assert!(matches!(pkg.marked(), Marked::Keep));

		let (_, pkg_set) = cache.sort_changes(HashSet::new()).unwrap();
		let reinstall = pkg_set.get(&Operation::Reinstall).unwrap();

		assert_eq!(reinstall.len(), 1);
		assert_eq!(reinstall[0].name, "nala-reinstall-test");
	}
}
