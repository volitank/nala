use anyhow::Result;
use rust_apt::cache::Upgrade;
use rust_apt::new_cache;

use crate::libnala::sudo_check;
use crate::{debug, Config};

pub async fn upgrade(config: &Config, upgrade_type: Upgrade) -> Result<()> {
	sudo_check(config)?;
	let cache = new_cache!()?;

	debug!("Running Upgrade: {upgrade_type:?}");
	cache.upgrade(upgrade_type)?;

	crate::summary::commit(cache, config).await
}
