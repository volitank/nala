use std::fmt::Write;
use std::path::Path;

use anyhow::{bail, Result};
use rust_apt::Version;
use serde::Serialize;
use sha2::{Digest, Sha256, Sha512};
use tokio::fs;
use tokio::io::AsyncReadExt;

use crate::config::{color, Theme};

/// Return the hash_type and the hash_value to be used.
pub fn get_hash(version: &Version) -> Result<HashSum> {
	// From Debian's requirements we are not to use these for security checking.
	// https://wiki.debian.org/DebianRepository/Format#MD5Sum.2C_SHA1.2C_SHA256
	// Clients may not use the MD5Sum and SHA1 fields for security purposes,
	// and must require a SHA256 or a SHA512 field.
	// hashes = ('SHA512', 'SHA256', 'SHA1', 'MD5')

	for hash_type in ["sha512", "sha256"] {
		if let Some(hash) = version.hash(hash_type) {
			return HashSum::from_str(hash_type, hash);
		}
	}

	bail!(
		"{} {} can't be checked for integrity.\nThere are no hashes available for this package.",
		color::color!(Theme::Notice, version.parent().name()),
		color::color!(Theme::Notice, version.version()),
	);
}

pub fn get_hasher(hash_type: &str) -> Result<Box<dyn digest::DynDigest + Send>> {
	Ok(match hash_type {
		"sha512" => Box::new(Sha512::new()),
		"sha256" => Box::new(Sha256::new()),
		anything_else => bail!("Hash Type: {anything_else} is not supported"),
	})
}

pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
	let mut hash = String::new();
	for byte in bytes {
		write!(&mut hash, "{:02x}", byte).expect("Unable to write hash to string");
	}
	hash
}

#[derive(Debug, Serialize, PartialEq)]
pub enum HashSum {
	Sha512(String),
	Sha256(String),
}

impl HashSum {
	pub fn from_str_len(hash_type: usize, hash: String) -> Result<Self> {
		Ok(match hash_type {
			128 => Self::Sha512(hash),
			64 => Self::Sha256(hash),
			anything_else => bail!("Hash Type: {anything_else} is not supported"),
		})
	}

	pub fn from_str(hash_type: &str, hash: String) -> Result<Self> {
		Ok(match hash_type {
			"sha512" => Self::Sha512(hash),
			"sha256" => Self::Sha256(hash),
			anything_else => bail!("Hash Type: {anything_else} is not supported"),
		})
	}

	pub async fn from_path<P: AsRef<Path>>(path: P, hash_type: &str) -> Result<Self> {
		let mut hasher = get_hasher(hash_type)?;

		let mut file = fs::File::open(&path).await?;
		let mut buffer = [0u8; 4096];

		// Read the file in chunks and feed it to the hasher.
		loop {
			let bytes_read = file.read(&mut buffer).await?;
			if bytes_read == 0 {
				break;
			}
			hasher.update(&buffer[..bytes_read]);
		}

		Self::from_str(hash_type, bytes_to_hex_string(&hasher.finalize()))
	}

	pub fn str_type(&self) -> &'static str {
		match self {
			Self::Sha512(_) => "sha512",
			Self::Sha256(_) => "sha256",
		}
	}
}
