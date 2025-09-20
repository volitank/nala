pub mod downloader;
pub mod proxy;
pub mod uri;

use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub use downloader::Downloader;
use indexmap::IndexSet;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::Frame;
use tokio::sync::RwLock;
pub use uri::{Uri, UriFilter};

use crate::config::Config;
use crate::tui::progress::DisplayGroup;
use crate::tui::{borderless_area, Drawable};

pub struct DomainMap {
	map: Arc<RwLock<HashMap<String, IndexSet<String>>>>,
	cache: HashMap<String, IndexSet<String>>,
}

impl DomainMap {
	pub fn new() -> Self {
		Self {
			map: Arc::new(RwLock::new(HashMap::new())),
			cache: HashMap::new(),
		}
	}

	pub fn inner(&self) -> Arc<RwLock<HashMap<String, IndexSet<String>>>> { Arc::clone(&self.map) }

	pub async fn add(&self, domain: &str, pkg: &str) -> bool {
		let mut lock = self.map.write().await;

		let entry = lock.entry(domain.to_string()).or_default();

		if entry.len() < 3 {
			entry.insert(pkg.to_string());
			return true;
		}
		return false;
	}

	pub async fn remove(&self, domain: &str, pkg: &str) {
		let mut lock = self.map.write().await;
		if let Some(pkgs) = lock.get_mut(domain) {
			pkgs.shift_remove(pkg);
			if pkgs.is_empty() {
				lock.remove(domain);
			}
		}
	}
}

impl Clone for DomainMap {
	fn clone(&self) -> Self {
		Self {
			map: self.inner(),
			cache: self.cache.clone(),
		}
	}
}

impl Deref for DomainMap {
	type Target = Arc<RwLock<HashMap<String, IndexSet<String>>>>;

	fn deref(&self) -> &Self::Target { &self.map }
}

pub struct DomainWidget {
	snapshot: HashMap<String, IndexSet<String>>,
}

impl DomainWidget {
	pub async fn new(map: &RwLock<HashMap<String, IndexSet<String>>>) -> Self {
		let guard = map.read().await;
		let snapshot = guard.clone();
		drop(guard);
		Self { snapshot }
	}
}

impl Drawable for DomainWidget {
	fn draw(&self, config: &Config, f: &mut Frame, area: Rect) {
		let inner = borderless_area(f, area, "Mirrors:");
		// if inner.width == 0 || inner.height == 0 {
		//     return;
		// }

		let mut groups: Vec<DisplayGroup> = Vec::with_capacity(self.snapshot.len() + 1);
		for (domain, downloads) in self.snapshot.iter() {
			let mut dg = DisplayGroup::new_no_value(domain);
			for (i, pkg) in downloads.iter().enumerate() {
				dg.insert((i + 1).to_string(), pkg.clone());
			}
			groups.push(dg);
		}

		if groups.is_empty() {
			return;
		}

		let constraints: Vec<Constraint> = vec![Constraint::Min(1); groups.len()];

		let slots = Layout::vertical(constraints).split(inner);
		for (dg, slot) in groups.iter().zip(slots.iter()) {
			dg.draw(config, f, *slot);
		}
	}

	fn height(&self) -> u16 {
		// One for Title
		let mut height = 1;
		for (_domain, pkgs) in self.snapshot.iter() {
			// One for Domain
			height += 1;
			height += pkgs.len()
		}
		height as u16
	}
}
