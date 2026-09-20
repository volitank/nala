use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use reqwest::{Client, RequestBuilder, Url};

use crate::config::Config;

/// Parsed APT auth files, kept separate to preserve first-match ordering per
/// file.
#[derive(Clone, Default)]
pub(super) struct AuthConf(Arc<[Vec<AuthEntry>]>);

struct AuthEntry {
	machine: String,
	login: String,
	password: String,
}

impl AuthConf {
	pub(super) fn load(config: &Config) -> Self {
		let mut paths = vec![PathBuf::from(
			config.apt.file("Dir::Etc::netrc", "/etc/apt/auth.conf"),
		)];
		let parts = PathBuf::from(
			config
				.apt
				.dir("Dir::Etc::netrcparts", "/etc/apt/auth.conf.d/"),
		);

		if let Ok(entries) = fs::read_dir(parts) {
			let mut part_paths: Vec<_> = entries
				.flatten()
				.map(|entry| entry.path())
				.filter(|path| {
					path.extension()
						.is_some_and(|extension| extension == "conf")
				})
				.collect();
			part_paths.sort();
			paths.extend(part_paths);
		}

		// Missing auth files are normal; APT also continues without
		// credentials.
		Self(
			paths
				.into_iter()
				.filter_map(|path| fs::read_to_string(path).ok())
				.map(|contents| parse(&contents))
				.collect(),
		)
	}

	pub(super) fn get(&self, client: &Client, url: &str) -> RequestBuilder {
		let request = client.get(url);
		let Ok(url) = Url::parse(url) else {
			return request;
		};
		let Some((login, password)) = self.credentials(&url) else {
			return request;
		};

		request.basic_auth(login, Some(password))
	}

	fn credentials(&self, url: &Url) -> Option<(&str, &str)> {
		if !url.username().is_empty() || url.password().is_some() {
			return None;
		}

		for file in self.0.iter() {
			let Some(entry) = file.iter().find(|entry| entry.matches(url)) else {
				continue;
			};
			if !entry.login.is_empty() || !entry.password.is_empty() {
				return Some((&entry.login, &entry.password));
			}
		}

		None
	}
}

impl AuthEntry {
	fn matches(&self, url: &Url) -> bool {
		let (protocol, machine) = self
			.machine
			.split_once("://")
			.map_or((None, self.machine.as_str()), |(protocol, machine)| {
				(Some(protocol), machine)
			});

		if protocol.is_some_and(|protocol| protocol != url.scheme())
			|| (protocol.is_none() && !matches!(url.scheme(), "https" | "tor+https"))
		{
			return false;
		}

		let Some(host) = url.host_str() else {
			return false;
		};
		let authority = url
			.port()
			.map_or_else(|| host.to_string(), |port| format!("{host}:{port}"));

		if machine.contains('/') {
			return format!("{authority}{}", url.path()).starts_with(machine);
		}

		machine == authority || (url.port().is_some() && machine == host)
	}
}

fn parse(contents: &str) -> Vec<AuthEntry> {
	let mut entries = Vec::new();
	let mut current: Option<AuthEntry> = None;
	let mut tokens = contents.split_whitespace();

	while let Some(token) = tokens.next() {
		match token {
			"machine" => {
				if let Some(entry) = current.take() {
					entries.push(entry);
				}
				current = tokens.next().map(|machine| AuthEntry {
					machine: machine.to_string(),
					login: String::new(),
					password: String::new(),
				});
			},
			"login" => {
				if let Some(entry) = current.as_mut()
					&& let Some(login) = tokens.next()
				{
					entry.login = login.to_string();
				}
			},
			"password" => {
				if let Some(entry) = current.as_mut()
					&& let Some(password) = tokens.next()
				{
					entry.password = password.to_string();
				}
			},
			_ => {},
		}
	}

	if let Some(entry) = current {
		entries.push(entry);
	}
	entries
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn matches_apt_auth_entries() {
		let auth = AuthConf(Arc::from([parse(
			"machine example.net login secure password all-ports\n\
			 machine https://example.org/private login path password secret\n\
			 machine http://example.org:8080/public login plain password explicit",
		)]));

		for (url, expected) in [
			(
				"https://example.net:8443/pkg",
				Some(("secure", "all-ports")),
			),
			("http://example.net/pkg", None),
			("https://example.org/private/pkg", Some(("path", "secret"))),
			("https://example.org/public/pkg", None),
			(
				"http://example.org:8080/public/pkg",
				Some(("plain", "explicit")),
			),
		] {
			assert_eq!(auth.credentials(&Url::parse(url).unwrap()), expected);
		}

		let request = auth
			.get(&Client::new(), "https://example.net/pkg")
			.build()
			.unwrap();
		assert_eq!(
			request.headers()["authorization"],
			"Basic c2VjdXJlOmFsbC1wb3J0cw=="
		);
	}
}
