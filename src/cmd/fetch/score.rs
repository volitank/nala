use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use reqwest::Client;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::Duration;

use crate::config::Config;
use crate::progress::Progress;
use crate::terminal::poll_exit_event;
use crate::t;

pub(super) async fn score_mirrors(
	config: &Config,
	mirrors: HashSet<String>,
	release: &str,
) -> Result<Vec<(String, u128)>> {
	let mut pb = Progress::new(config, false)?;
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
			pb.display_mut()
				.push_str(format!("{} ", t!("progress-finished")), response.0.to_string());
			scores.push(response)
		}
		pb.inc(1);
		pb.render()?;
		if poll_exit_event()? {
			pb.clean_up()?;
			std::process::exit(1);
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
	client
		.get(format!("{url}/dists/{release}/Release"))
		.send()
		.await?
		.error_for_status()?
		.bytes()
		.await?;
	let after = before.elapsed().as_millis();
	Ok((url, after))
}

fn score_url(url: &str, https_only: bool) -> String {
	if https_only {
		return url.replacen("http://", "https://", 1);
	}
	url.to_string()
}
