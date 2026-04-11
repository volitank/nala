pub mod downloader;
pub mod proxy;
pub mod uri;

use std::collections::HashMap;
use std::sync::Arc;

pub use downloader::{download, Downloader};
use indexmap::IndexSet;
use tokio::sync::RwLock;
pub use uri::{Uri, UriFilter};

use crate::progress::ProgressPanel;

#[derive(Clone, Default)]
pub(crate) struct DomainMap {
	map: Arc<RwLock<HashMap<String, IndexSet<String>>>>,
}

impl DomainMap {
	pub(crate) fn new() -> Self {
		Self {
			map: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	pub(crate) async fn add(&self, domain: &str, pkg: &str) -> bool {
		let mut lock = self.map.write().await;
		let entry = lock.entry(domain.to_string()).or_default();

		if entry.len() < 3 {
			entry.insert(pkg.to_string());
			return true;
		}

		false
	}

	pub(crate) async fn remove(&self, domain: &str, pkg: &str) {
		let mut lock = self.map.write().await;
		if let Some(pkgs) = lock.get_mut(domain) {
			pkgs.shift_remove(pkg);
			if pkgs.is_empty() {
				lock.remove(domain);
			}
		}
	}

	pub(crate) async fn panels(&self) -> Vec<ProgressPanel> {
		let snapshot = self.map.read().await.clone();
		let mut rows = snapshot.into_iter().collect::<Vec<_>>();
		rows.sort_by(|a, b| a.0.cmp(&b.0));

		rows.into_iter()
			.map(|(domain, downloads)| {
				let mut panel = ProgressPanel::new(domain);
				for pkg in downloads {
					panel.push(pkg);
				}
				panel
			})
			.collect()
	}
}
