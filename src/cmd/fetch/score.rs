use std::collections::HashSet;
use std::sync::Arc;

use anyhow::{Result, bail, ensure};
use reqwest::Client;
use rust_apt::tagfile::parse_tagfile;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::Duration;

use crate::config::Config;
use crate::progress::{Progress, ProgressMessage};
use crate::t;
use crate::terminal::poll_exit_event;

pub(super) async fn score_mirrors(
	config: &Config,
	mirrors: HashSet<String>,
	release: &str,
) -> Result<Vec<(String, u128)>> {
	let mut pb = Progress::mirror_score(config)?;
	pb.set_length(mirrors.len() as u64);

	let client = Client::builder()
		.timeout(Duration::from_secs(1))
		.retry(reqwest::retry::never())
		.pool_max_idle_per_host(0)
		.build()?;

	let limit = Arc::new(Semaphore::new(10));
	let mut set = JoinSet::new();
	for url in &mirrors {
		set.spawn(score_mirror(
			client.clone(),
			limit.clone(),
			config.get_bool("https_only", false),
			url.strip_suffix('/').unwrap_or(url).to_string(),
			release.to_string(),
		));
	}

	let mut scores = vec![];
	while let Some(res) = set.join_next().await {
		if let Ok(Ok(response)) = res {
			pb.set_message(ProgressMessage::new(
				format!("{} ", t!("progress-finished")),
				vec![response.0.to_string()],
			));
			scores.push(response)
		}
		pb.inc(1);
		pb.render()?;
		if poll_exit_event()? {
			pb.clean_up()?;
			set.shutdown().await;
			bail!("{}", t!("download-exit"));
		}
	}
	pb.clean_up()?;

	scores.sort_by_key(|k| k.1);
	Ok(scores)
}

async fn score_mirror(
	client: Client,
	limit: Arc<Semaphore>,
	https_only: bool,
	url: String,
	release: String,
) -> Result<(String, u128)> {
	let _permit = limit.acquire_owned().await?;
	let url = score_url(&url, https_only);

	let before = std::time::Instant::now();
	let body = client
		.get(format!("{url}/dists/{release}/Release"))
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?;
	let after = before.elapsed().as_millis();
	validate_release(&body)?;
	Ok((url, after))
}

fn validate_release(body: &[u8]) -> Result<()> {
	let sections = parse_tagfile(std::str::from_utf8(body)?)?;
	let [release] = sections.as_slice() else {
		bail!("invalid Release file")
	};

	ensure!(
		release.get("Architectures").is_some()
			&& release.get("Components").is_some()
			&& ["SHA256", "SHA512"]
				.iter()
				.any(|field| release.get(field).is_some()),
		"invalid Release file"
	);
	Ok(())
}

fn score_url(url: &str, https_only: bool) -> String {
	if https_only {
		return url.replacen("http://", "https://", 1);
	}
	url.to_string()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn release_validation_rejects_non_release_content() {
		let release = b"Origin: Debian\nSuite: stable\nArchitectures: amd64 arm64\nComponents: main\nSHA256:\n abc 123 file\n";
		let weak_release = b"Origin: Debian\nSuite: stable\nArchitectures: amd64 arm64\nComponents: main\nMD5Sum:\n abc 123 file\n";
		let html = b"<!doctype html>\n<html><body>Domain for sale</body></html>";
		let garbage = b"Title: Domain for sale\nDescription: This is valid RFC 822, but not a Release file.\n";

		assert!(validate_release(release).is_ok());
		assert!(validate_release(weak_release).is_err());
		assert!(validate_release(html).is_err());
		assert!(validate_release(garbage).is_err());
	}
}
