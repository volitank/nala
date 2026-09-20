mod auth;
pub mod downloader;
pub mod proxy;
pub mod uri;

use std::collections::VecDeque;
use std::sync::Arc;

pub use downloader::{Downloader, download};
use indexmap::{IndexMap, IndexSet};
use tokio::sync::{Mutex, oneshot};
pub use uri::{Uri, UriFilter};

pub(crate) const DOMAIN_CONNECTION_LIMIT: usize = 3;

#[derive(Default)]
struct Domain {
	active: IndexSet<String>,
	// Packages with no alternate mirror keep this domain's slots reserved.
	exclusive: IndexSet<String>,
}

struct Waiter {
	package: String,
	candidates: Vec<String>,
	ready: oneshot::Sender<String>,
}

#[derive(Default)]
struct DomainState {
	domains: IndexMap<String, Domain>,
	waiters: VecDeque<Waiter>,
	next: usize,
}

impl DomainState {
	fn reserve(&mut self, package: &str, candidates: &[String]) -> Option<String> {
		let len = candidates.len();
		if len == 0 {
			return None;
		}

		let start = self.next % len;
		self.next = self.next.wrapping_add(1);
		for offset in 0..len {
			let name = &candidates[(start + offset) % len];
			let Some(domain) = self.domains.get_mut(name) else {
				continue;
			};
			let exclusive = domain.exclusive.contains(package);
			if domain.active.len() >= DOMAIN_CONNECTION_LIMIT
				|| (!exclusive && !domain.exclusive.is_empty())
			{
				continue;
			}

			domain.active.insert(package.to_string());
			domain.exclusive.shift_remove(package);
			return Some(name.clone());
		}

		None
	}

	fn dispatch(&mut self) {
		loop {
			let mut assigned = false;
			let pending = self.waiters.len();
			for _ in 0..pending {
				let waiter = self.waiters.pop_front().expect("pending waiter exists");
				let Some(domain) = self.reserve(&waiter.package, &waiter.candidates) else {
					self.waiters.push_back(waiter);
					continue;
				};

				if waiter.ready.send(domain.clone()).is_err()
					&& let Some(domain) = self.domains.get_mut(&domain)
				{
					domain.active.shift_remove(&waiter.package);
				}
				assigned = true;
				break;
			}
			if !assigned {
				break;
			}
		}
	}
}

#[derive(Clone, Default)]
pub(crate) struct DomainMap {
	state: Arc<Mutex<DomainState>>,
}

impl DomainMap {
	pub(crate) async fn register(&self, package: &str, candidates: &[String]) {
		let mut state = self.state.lock().await;
		for domain in candidates {
			state.domains.entry(domain.clone()).or_default();
		}
		if candidates.len() == 1 {
			state.domains[&candidates[0]]
				.exclusive
				.insert(package.to_string());
		}
	}

	pub(crate) async fn acquire(&self, package: &str, candidates: &[String]) -> Option<String> {
		if candidates.is_empty() {
			return None;
		}

		let receiver = {
			let mut state = self.state.lock().await;
			if let Some(domain) = state.reserve(package, candidates) {
				return Some(domain);
			}

			let (ready, receiver) = oneshot::channel();
			state.waiters.push_back(Waiter {
				package: package.to_string(),
				candidates: candidates.to_vec(),
				ready,
			});
			receiver
		};

		receiver.await.ok()
	}

	pub(crate) async fn cancel(&self, package: &str) {
		let mut state = self.state.lock().await;
		for domain in state.domains.values_mut() {
			domain.exclusive.shift_remove(package);
		}
		state.waiters.retain(|waiter| waiter.package != package);
		state.dispatch();
	}

	pub(crate) async fn remove(&self, domain: &str, pkg: &str) {
		let mut state = self.state.lock().await;
		if state
			.domains
			.get_mut(domain)
			.is_some_and(|domain| domain.active.shift_remove(pkg))
		{
			state.dispatch();
		}
	}

	pub(crate) async fn active(&self) -> Vec<(String, usize)> {
		self.state
			.lock()
			.await
			.domains
			.iter()
			.map(|(name, domain)| (name.clone(), domain.active.len()))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use super::{DOMAIN_CONNECTION_LIMIT, DomainMap};

	#[tokio::test]
	async fn scarce_domains_are_reserved_for_exclusive_downloads() {
		let domains = DomainMap::default();
		let scarce = vec!["scarce.example".into()];
		let flexible = vec!["scarce.example".into(), "mirror.example".into()];
		domains.register("exclusive", &scarce).await;
		domains.register("flexible", &flexible).await;

		assert_eq!(
			domains.acquire("flexible", &flexible).await.as_deref(),
			Some("mirror.example")
		);
		assert_eq!(
			domains.acquire("exclusive", &scarce).await.as_deref(),
			Some("scarce.example")
		);

		assert_eq!(
			domains.active().await,
			vec![("scarce.example".into(), 1), ("mirror.example".into(), 1)]
		);
	}

	#[tokio::test]
	async fn released_slots_go_to_eligible_waiters() {
		let domains = DomainMap::default();
		let one = vec!["one.example".into()];
		let two = vec!["two.example".into()];

		for index in 0..DOMAIN_CONNECTION_LIMIT {
			let package = format!("one-{index}");
			domains.register(&package, &one).await;
			assert_eq!(domains.acquire(&package, &one).await, Some(one[0].clone()));
			let package = format!("two-{index}");
			domains.register(&package, &two).await;
			assert_eq!(domains.acquire(&package, &two).await, Some(two[0].clone()));
		}

		domains.register("waiting-one", &one).await;
		let waiting = domains.clone();
		let one_candidates = one.clone();
		let mut waiting_one =
			tokio::spawn(async move { waiting.acquire("waiting-one", &one_candidates).await });
		tokio::task::yield_now().await;

		domains.register("waiting-two", &two).await;
		let waiting = domains.clone();
		let two_candidates = two.clone();
		let waiting_two =
			tokio::spawn(async move { waiting.acquire("waiting-two", &two_candidates).await });
		tokio::task::yield_now().await;

		domains.remove("two.example", "two-0").await;
		assert_eq!(
			tokio::time::timeout(Duration::from_secs(1), waiting_two)
				.await
				.unwrap()
				.unwrap()
				.as_deref(),
			Some("two.example")
		);
		assert_eq!(
			domains.active().await,
			vec![
				("one.example".into(), DOMAIN_CONNECTION_LIMIT),
				("two.example".into(), DOMAIN_CONNECTION_LIMIT),
			]
		);

		domains.remove("one.example", "one-0").await;
		assert_eq!(
			tokio::time::timeout(Duration::from_secs(1), &mut waiting_one)
				.await
				.unwrap()
				.unwrap()
				.as_deref(),
			Some("one.example")
		);
	}
}
