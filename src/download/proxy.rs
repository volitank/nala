use std::collections::HashMap;
use std::process::Command;
use std::sync::Mutex;

use anyhow::{Context, Result, bail};
use tokio::sync::mpsc;

use super::downloader::Message;
use crate::config::Config;

#[derive(Clone, Debug, Eq, PartialEq)]
enum ProxySetting {
	Proxy(reqwest::Url),
	Direct,
}

impl ProxySetting {
	fn from_apt(value: &str) -> Result<Self> {
		if value.eq_ignore_ascii_case("direct") || value.eq_ignore_ascii_case("false") {
			return Ok(Self::Direct);
		}
		let proxy = reqwest::Url::parse(value)?;
		if !matches!(proxy.scheme(), "http" | "https" | "socks5h") {
			bail!("Unsupported proxy scheme '{}'", proxy.scheme());
		}
		Ok(Self::Proxy(proxy))
	}

	fn proxy(&self) -> Option<reqwest::Url> {
		match self {
			Self::Proxy(proxy) => Some(proxy.clone()),
			Self::Direct => None,
		}
	}
}

fn auto_detect_proxy(
	command: &str,
	proto: &str,
	url: &reqwest::Url,
) -> Result<Option<ProxySetting>> {
	let output = Command::new(command)
		.arg(url.as_str())
		.output()
		.with_context(|| format!("Failed to execute proxy auto-detect command '{command}'"))?;

	if !output.status.success() {
		bail!(
			"Proxy auto-detect command '{command}' exited with {}",
			output.status
		);
	}
	parse_auto_detect_output(proto, &output.stdout)
}

fn parse_auto_detect_output(proto: &str, output: &[u8]) -> Result<Option<ProxySetting>> {
	let Some(line) = std::str::from_utf8(output)?.lines().next() else {
		return Ok(None);
	};
	let proxy = line.trim();
	if proxy.is_empty() {
		return Ok(None);
	}
	if proxy == "DIRECT" {
		return Ok(Some(ProxySetting::Direct));
	}

	let proxy = reqwest::Url::parse(proxy)?;
	if !matches!(proxy.scheme(), "http" | "https" | "socks5h") {
		bail!("Proxy auto-detect command returned incompatible proxy '{proxy}' for {proto}");
	}
	Ok(Some(ProxySetting::Proxy(proxy)))
}

pub fn build_proxy(config: &Config, tx: mpsc::UnboundedSender<Message>) -> Result<reqwest::Proxy> {
	let mut map: HashMap<String, ProxySetting> = HashMap::new();
	let mut auto_detect = HashMap::new();

	for proto in ["http", "https"] {
		let modern = format!("Acquire::{proto}::Proxy-Auto-Detect");
		let legacy = format!("Acquire::{proto}::ProxyAutoDetect");
		if let Some(command) = config.apt.get(&modern).or_else(|| config.apt.get(&legacy)) {
			auto_detect.insert(proto.to_string(), command);
		}

		if let Some(proxy_config) = config.apt.tree(&format!("Acquire::{proto}::Proxy")) {
			// Check first for a proxy for everything
			if let Some(proxy) = proxy_config.value() {
				map.insert(proto.to_string(), ProxySetting::from_apt(&proxy)?);
			}

			// Check for specific domain proxies
			if let Some(child) = proxy_config.child() {
				for node in child {
					let (Some(domain), Some(proxy)) = (node.tag(), node.value()) else {
						continue;
					};

					map.insert(
						format!("{proto}://{domain}"),
						ProxySetting::from_apt(&proxy)?,
					);
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

	let debug = config.debug();
	let detected = Mutex::new(HashMap::<String, Option<ProxySetting>>::new());
	Ok(reqwest::Proxy::custom(move |url| {
		let domain = url.host_str()?;
		let key = format!("{}://{domain}", url.scheme());

		// An explicit host setting always takes precedence over auto-detection.
		if let Some(setting) = map.get(&key) {
			let proxy = setting.proxy();
			send_debug(&tx, debug, domain, proxy.as_ref());
			return proxy;
		}

		if let Some(command) = auto_detect.get(url.scheme()) {
			let proxy = detected
				.lock()
				.unwrap()
				.entry(key)
				.or_insert_with(|| match auto_detect_proxy(command, url.scheme(), url) {
					Ok(proxy) => proxy,
					Err(error) => {
						if debug {
							let _ = tx.send(Message::Debug(error.to_string()));
						}
						None
					},
				})
				.clone();

			if let Some(proxy) = proxy {
				let proxy = proxy.proxy();
				send_debug(&tx, debug, domain, proxy.as_ref());
				return proxy;
			}
		}

		if let Some(setting) = map.get(url.scheme()) {
			let proxy = setting.proxy();
			send_debug(&tx, debug, domain, proxy.as_ref());
			return proxy;
		}
		send_debug(&tx, debug, domain, None);
		None
	}))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn auto_detect_output_matches_apt() {
		assert_eq!(parse_auto_detect_output("http", b"").unwrap(), None);
		assert_eq!(
			parse_auto_detect_output("http", b"DIRECT\n").unwrap(),
			Some(ProxySetting::Direct)
		);
		assert_eq!(
			parse_auto_detect_output("http", b"http://proxy.example:3142\n").unwrap(),
			Some(ProxySetting::Proxy(
				reqwest::Url::parse("http://proxy.example:3142").unwrap()
			))
		);
		assert!(parse_auto_detect_output("http", b"ftp://proxy.example\n").is_err());
	}
}
