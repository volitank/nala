pub mod downloader;
pub mod proxy;
pub mod uri;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use downloader::{Downloader, download};
use indexmap::{IndexMap, IndexSet};
use tokio::sync::futures::Notified;
use tokio::sync::{Notify, RwLock};
pub use uri::{Uri, UriFilter};

pub(crate) const DOMAIN_CONNECTION_LIMIT: usize = 3;

#[derive(Clone, Default)]
pub(crate) struct DomainMap {
	map: Arc<RwLock<IndexMap<String, IndexSet<String>>>>,
	available: Arc<Notify>,
	next: Arc<AtomicUsize>,
}

impl DomainMap {
	pub(crate) async fn register(&self, domains: impl IntoIterator<Item = String>) {
		let mut lock = self.map.write().await;
		for domain in domains {
			lock.entry(domain).or_default();
		}
	}

	pub(crate) async fn add(&self, domain: &str, pkg: &str) -> bool {
		let mut lock = self.map.write().await;
		let entry = lock.entry(domain.to_string()).or_default();

		if entry.len() < DOMAIN_CONNECTION_LIMIT {
			entry.insert(pkg.to_string());
			return true;
		}

		false
	}

	pub(crate) async fn remove(&self, domain: &str, pkg: &str) {
		let mut lock = self.map.write().await;
		let removed = lock
			.get_mut(domain)
			.is_some_and(|pkgs| pkgs.shift_remove(pkg));
		drop(lock);

		if removed {
			self.available.notify_one();
		}
	}

	pub(crate) fn notified(&self) -> Notified<'_> { self.available.notified() }

	/// Rotates which mirror gets first chance at the next available slot.
	pub(crate) fn start_index(&self, candidates: usize) -> usize {
		self.next.fetch_add(1, Ordering::Relaxed) % candidates
	}

	pub(crate) async fn active(&self) -> Vec<(String, usize)> {
		self.map
			.read()
			.await
			.iter()
			.map(|(domain, packages)| (domain.clone(), packages.len()))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::{DOMAIN_CONNECTION_LIMIT, DomainMap};

	#[tokio::test]
	async fn registered_domains_stay_visible_when_idle() {
		let domains = DomainMap::default();
		domains
			.register(["one.example".into(), "two.example".into()])
			.await;

		for package in 0..DOMAIN_CONNECTION_LIMIT {
			assert!(
				domains
					.add("one.example", &format!("package-{package}"))
					.await
			);
		}
		assert!(!domains.add("one.example", "one-too-many").await);
		let waiting = domains.clone();
		let waiter = tokio::spawn(async move { waiting.notified().await });
		tokio::task::yield_now().await;
		domains.remove("one.example", "package-0").await;
		tokio::time::timeout(std::time::Duration::from_secs(1), waiter)
			.await
			.unwrap()
			.unwrap();

		assert_eq!(
			domains.active().await,
			vec![("one.example".into(), 2), ("two.example".into(), 0)]
		);
	}
}
