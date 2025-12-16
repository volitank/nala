use std::collections::HashMap;
use std::io::Cursor;

use anyhow::{anyhow, Result};
use ar::Archive;
use rust_apt::tagfile;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::Archive as Tarchive;
use tokio::fs;

use super::{Decompress, Reader};

#[derive(Debug, Serialize, Deserialize)]
pub struct DebFile {
	pub path: String,
	/// Parsed control fields from the archive's control tarball
	pub control: HashMap<String, String>,
	/// SHA256 of the full .deb archive
	pub hash: String,
}

impl DebFile {
	pub async fn new(path: String) -> Result<DebFile> {
		let data = fs::read(&path).await?;
		let mut ar = Archive::new(data.as_slice());
		let hash = format!("{:x}", Sha256::digest(&data));

		let mut control: Option<HashMap<String, String>> = None;
		while let Some(res) = ar.next_entry() {
			let mut entry = res?;
			let tarball = std::str::from_utf8(entry.header().identifier())?;
			if !tarball.starts_with("control.tar") {
				continue;
			}

			let mut tar = Tarchive::new(Cursor::new(entry.read_vec()?.decompress().await?));
			for file in tar.entries()? {
				let mut entry = file?;
				if entry.path()?.as_os_str() != "./control" {
					continue;
				}

				control = tagfile::parse_tagfile(&entry.read_string()?)?
					.into_iter()
					.next()
					.map(Into::into);
				break;
			}

			if control.is_some() {
				break;
			}
		}

		let control = control.ok_or_else(|| anyhow!("control file not found in {path}"))?;

		Ok(DebFile {
			path,
			control,
			hash,
		})
	}

	pub fn name(&self) -> &str {
		self.control
			.get("Package")
			.map(String::as_str)
			.unwrap_or("")
	}

	pub fn version(&self) -> &str {
		self.control
			.get("Version")
			.map(String::as_str)
			.unwrap_or("")
	}
}
