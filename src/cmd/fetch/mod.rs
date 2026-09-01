mod mirrors;
mod output;
mod release;
mod score;
mod sources;

use std::collections::HashSet;

use anyhow::{bail, Result};

use crate::config::Config;
use crate::terminal::{use_tui, TerminalGuard};
use crate::util::sudo_check;
use crate::{debug, tui};
use crate::t;

fn selected_countries(config: &Config) -> Option<HashSet<String>> {
	let values = config.countries()?;
	let mut countries = HashSet::new();

	for value in values {
		countries.insert(value.to_uppercase());
	}

	Some(countries)
}

fn remove_existing_sources(net_select: &mut HashSet<String>, sources: &HashSet<String>) {
	let mut remove = HashSet::new();

	for mirror in net_select.iter() {
		for source in sources {
			if mirror.contains(source) {
				remove.insert(mirror.to_string());
			}
		}
	}

	net_select.retain(|n| !remove.contains(n));
}

fn automatic_mirrors(scored: Vec<(String, u128)>, count: usize) -> Vec<String> {
	scored
		.into_iter()
		.take(count)
		.map(|(mirror, _)| mirror)
		.collect()
}

/// The entry point for the `fetch` command.
pub async fn fetch(config: &Config) -> Result<()> {
	sudo_check(config)?;

	let (distro, release, keyring) = release::detect_release(config)?;
	debug!("Detected '{distro}:{release}'");

	let component = release::get_component(config, &distro)?;
	debug!("Initial component '{component}'");

	let countries = selected_countries(config);

	// Get the current sources on disk to not create duplicates
	let sources = sources::parse_sources(config)?;
	debug!("Sources on disk {sources:#?}");

	// Get the mirrors
	let mut net_select = mirrors::fetch_mirrors(config, &countries, &distro).await?;
	debug!("NetSelect size '{}'", net_select.len());

	// Remove domains that are already defined on disk
	remove_existing_sources(&mut net_select, &sources);
	debug!("NetSelect Dedupe Size '{}'", net_select.len());

	// Score the mirrors
	let scored = score::score_mirrors(config, net_select, &release).await?;
	debug!("Scored Mirrors '{}'", scored.len());

	if scored.is_empty() {
		bail!("{}", t!("fetch-no-mirrors"))
	}

	// Only run the TUI if --auto is not on and a TUI is available.
	let chosen = if let Some(count) = config.auto() {
		debug!("Auto mode, not starting TUI");
		automatic_mirrors(scored, usize::from(count))
	} else if !use_tui(config) {
		debug!("Plain mode, not starting TUI");
		automatic_mirrors(scored, 3)
	} else {
		debug!("Interactive mode, starting TUI");
		let mut terminal = TerminalGuard::new()?;
		let app = tui::fetch::App::new(config, scored);
		app.run(terminal.terminal_mut())?
	};

	if chosen.is_empty() {
		bail!("{}", t!("fetch-none-selected"))
	}

	output::write_nala_sources(config, &chosen, component, &release, &keyring).await
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn automatic_selection_keeps_the_fastest_requested_mirrors() {
		let scored = vec![
			("first".to_string(), 1),
			("second".to_string(), 2),
			("third".to_string(), 3),
			("fourth".to_string(), 4),
		];

		assert_eq!(automatic_mirrors(scored, 3), ["first", "second", "third"]);
	}
}
