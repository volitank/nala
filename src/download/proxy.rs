use std::collections::HashMap;

use anyhow::Result;
use tokio::sync::mpsc;

use super::downloader::Message;
use crate::config::Config;

#[derive(Debug, Eq, Hash, PartialEq)]
enum Proto {
	Http(reqwest::Url),
	Https(reqwest::Url),
	None,
}

impl Proto {
	fn new(proto: &str, domain: reqwest::Url) -> Self {
		match proto {
			"http" => Self::Http(domain),
			"https" => Self::Https(domain),
			_ => panic!("Protocol '{proto}' is not supported!"),
		}
	}

	fn maybe_proxy(&self, url: &reqwest::Url) -> Option<reqwest::Url> {
		match (self, url.scheme()) {
			// The protocol and proxy config match.
			(Proto::Http(proxy), "http") => Some(proxy.clone()),
			(Proto::Https(proxy), "https") => Some(proxy.clone()),

			// The protocol and config doesn't match.
			(Proto::Http(_), "https") => None,
			(Proto::Https(_), "http") => None,

			// For other URL schemes such as socks or ftp
			// We will just proxy them
			(Proto::Http(proxy), _) => Some(proxy.clone()),
			(Proto::Https(proxy), _) => Some(proxy.clone()),
			// This one should never actually be reached
			(Proto::None, _) => None,
		}
	}

	/// Used to get the default for all http/https if configured
	fn proxy(&self) -> Option<reqwest::Url> {
		match self {
			Proto::Http(proxy) => Some(proxy.clone()),
			Proto::Https(proxy) => Some(proxy.clone()),
			Proto::None => None,
		}
	}
}

pub fn build_proxy(config: &Config, tx: mpsc::UnboundedSender<Message>) -> Result<reqwest::Proxy> {
	let mut map: HashMap<String, Proto> = HashMap::new();

	for proto in ["http", "https"] {
		if let Some(proxy_config) = config.apt.tree(&format!("Acquire::{proto}::Proxy")) {
			// Check first for a proxy for everything
			if let Some(proxy) = proxy_config.value() {
				map.insert(
					proto.to_string(),
					Proto::new(proto, reqwest::Url::parse(&proxy)?),
				);
			}

			// Check for specific domain proxies
			if let Some(child) = proxy_config.child() {
				for node in child {
					let (Some(domain), Some(proxy)) = (node.tag(), node.value()) else {
						continue;
					};

					let lower = proxy.to_lowercase();
					if ["direct", "false"].contains(&lower.as_str()) {
						map.insert(domain, Proto::None);
						continue;
					}
					map.insert(domain, Proto::new(proto, reqwest::Url::parse(&proxy)?));
				}
			}
		}
	}

	/// Helper function to make debug messages cleaner.
	fn send_debug(
		tx: &mpsc::UnboundedSender<Message>,
		debug: bool,
		domain: &str,
		proxy: Option<&reqwest::Url>,
	) {
		if debug {
			let message = if let Some(proxy) = proxy {
				format!("Proxy for '{domain}' is '{proxy}'")
			} else {
				format!("'{domain}' Proxy is None")
			};

			tx.send(Message::Debug(message))
				.unwrap_or_else(|e| eprintln!("Error: {e}"));
		}
	}

	fn get_proxy(
		map: &HashMap<String, Proto>,
		domain: &str,
		url: &reqwest::Url,
	) -> Option<reqwest::Url> {
		// Returns None if the domain is not in the map.
		// But checking for a default is still required.
		if let Some(proto) = map.get(domain) {
			if proto == &Proto::None {
				// This domain is specifically set to not use a proxy.
				return None;
			}

			// We have to check the maybe proxy as it is based on
			// the protocol of the URL matching the config.
			// The proxy function below will not account for that.
			if let Some(proxy) = proto.maybe_proxy(url) {
				return Some(proxy);
			}
		}

		// Check for http/s default proxy.
		map.get(url.scheme())?.proxy()
	}

	let debug = config.debug();
	Ok(reqwest::Proxy::custom(move |url| {
		let domain = url.host_str()?;

		if let Some(proxy) = get_proxy(&map, domain, url) {
			send_debug(&tx, debug, domain, Some(&proxy));
			return Some(proxy);
		}
		send_debug(&tx, debug, domain, None);
		None
	}))
}
