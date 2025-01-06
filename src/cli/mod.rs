mod clean;
mod fetch;
mod history;
mod install;
mod list;
pub mod parser;
mod show;
mod update;
mod upgrade;

use anyhow::{bail, Result};
pub use clean::clean;
pub use fetch::fetch;
pub use history::history;
pub use install::mark_cli_pkgs;
pub use parser::{Commands, NalaParser};
use rust_apt::cache::Upgrade;
use rust_apt::{new_cache, PackageSort, Version};
pub use show::show;
pub use update::update;
pub use upgrade::upgrade;

use crate::config::{Config, Paths, Theme};
use crate::deb::DebFile;
use crate::download::Downloader;
use crate::libnala::{sudo_check, Operation};
use crate::{color, glob, primary, tui, warn};

impl Commands {
	pub async fn run(&self, config: &mut Config) -> Result<()> {
		match self {
			Commands::List(_) | Commands::Search(_) => {
				let cache = new_cache!()?;
				list::list_packages(
					config,
					if config.command == "search" {
						glob::regex_pkgs(config, &cache)?.found().collect()
					} else if config.pkg_names().is_ok() {
						glob::pkgs_with_modifiers(config.pkg_names()?, config, &cache)?
							.found()
							.collect()
					} else {
						cache.packages(&glob::get_sorter(config)).collect()
					},
				)?;
			},
			Commands::Show(_) => show(config)?,
			Commands::Clean(_) => clean(config)?,
			Commands::Download(_) => {
				// Set download directory to the cwd.
				config.apt.set(Paths::Archive.path(), "./");

				let mut downloader = Downloader::new(config)?;
				let mut not_found = vec![];

				let cache = new_cache!()?;
				let pkg_names = config.pkg_names()?;
				let archive = config.get_path(&Paths::Archive);
				for name in &pkg_names {
					if let Some(pkg) = cache.get(name) {
						let versions: Vec<Version> = pkg.versions().collect();
						for version in &versions {
							if version.is_downloadable() {
								downloader.add_version(version, &archive).await?;
								break;
							}
							warn!(
								"Can't find a source to download version '{}' of '{}'",
								version.version(),
								pkg.fullname(false)
							);
						}
					} else {
						not_found.push(color!(Theme::Notice, name).to_string());
					}
				}

				if !not_found.is_empty() {
					for pkg in &not_found {
						color!(Theme::Error, &format!("{pkg} not found"));
					}
					bail!("Some packages were not found.");
				}

				let finished = downloader.run(config, true).await?;
				println!("Downloads Complete:");
				for uri in finished {
					println!(
						"  {} was written to {}",
						primary!(&uri.filename),
						primary!(&uri.archive.to_string_lossy()),
					)
				}
			},
			Commands::History(_) => history(config).await?,
			Commands::Fetch(_) => fetch(config).await?,
			Commands::Update(_) => update(config).await?,
			Commands::Upgrade(_) => {
				upgrade(
					config,
					// SafeUpgrade takes precedence.
					if config.get_bool("safe", false) {
						Upgrade::SafeUpgrade
					} else if config.get_no_bool("full", false) {
						Upgrade::FullUpgrade
					} else {
						Upgrade::Upgrade
					},
				)
				.await?
			},
			Commands::Install(_) => mark_cli_pkgs(config, Operation::Install).await?,
			Commands::Remove(_) => mark_cli_pkgs(config, Operation::Remove).await?,
			Commands::AutoRemove(_) => {
				sudo_check(config)?;
				crate::summary::commit(new_cache!()?, config).await?;
			},
			Commands::System(_) => {
				// This downloads all of the pkgs into the archives directory
				// let cache = rust_apt::new_cache!()?;
				// println!("Cache Total Pkgs: {}", cache.iter().count());

				// let mut downloader = Downloader::new(config)?;

				// let versions = cache
				// 	.iter()
				// 	.filter_map(|p| {
				// 		let v = p.installed()?;
				// 		if v.is_downloadable() {
				// 			Some(v)
				// 		} else {
				// 			None
				// 		}
				// 	})
				// 	.collect::<Vec<_>>();

				// for ver in &versions {
				// 	downloader.add_version(ver, config).await?;
				// }

				// downloader.run(config, false).await?;

				let archive = config.get_path(&Paths::Archive);
				let mut debs = vec![];
				for entry in std::fs::read_dir(archive)? {
					let entry = entry?;
					let metadata = entry.metadata()?;

					let path = entry.path();

					// If it's a directory, recurse into it
					if metadata.is_dir() {
						continue;
					}

					debs.push(path.to_string_lossy().to_string())
				}

				let cache = new_cache!(&debs)?;
				let filtered_pkgs = cache
					.packages(&PackageSort::default().installed())
					.filter_map(|pkg| {
						let version = pkg.installed()?;
						let file = version
							.version_files()
							.filter_map(|vf| {
								let pf = vf.package_file();
								// Instead of archive we could match the filename
								// pf.filename().unwrap().contains(Paths::Archive.default_path())
								if pf.archive()? == "local-deb" {
									Some(pf.filename()?.to_string())
								} else {
									None
								}
							})
							.next()?;
						Some((version, file))
					})
					.collect::<Vec<_>>();

				let mut pb = tui::NalaProgressBar::new(config, true)?;
				let mut set = tokio::task::JoinSet::new();
				for (_, file) in filtered_pkgs {
					set.spawn(DebFile::new(file));
				}

				let files = pb.join(set).await?;
				for file in files {
					file.store().await?;
				}
			},
		}
		Ok(())
	}
}
