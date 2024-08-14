use anyhow::Result;
use rust_apt::new_cache;

use crate::config::Config;
use crate::dpkg;
use crate::upgrade::auto_remover;
use crate::util::sudo_check;

#[tokio::main]
pub async fn install(config: &Config) -> Result<()> {
	sudo_check(config)?;

	config.apt.set("Dpkg::Use-Pty", "0");
	let cache = new_cache!()?;

	// let pkg = cache.get("nala").unwrap();
	// pkg.mark_reinstall(true);
	// pkg.mark_install(true, true);

	let pkg = cache.get("neofetch").unwrap();

	if pkg.is_installed() {
		pkg.mark_delete(true);
	} else {
		pkg.mark_install(true, true);
	}

	auto_remover(&cache);

	cache.resolve(false)?;

	dpkg::run_install(cache, config)?;

	Ok(())
}
