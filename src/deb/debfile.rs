use std::collections::HashMap;
use std::io::Cursor;

use anyhow::Result;
use ar::Archive;
use rust_apt::tagfile;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tar::Archive as Tarchive;
use tokio::io::AsyncWriteExt;

use super::{Decompress, Reader};
use crate::fs::AsyncFs;
use crate::tui::progress::ProgressItem;

#[derive(Debug, Serialize, Deserialize)]
pub enum FileType {
	/// The File will contain the Hash
	File(String),
	/// The Symlink will contain TODO: Idk
	Symlink(String),
	/// Simple Directory
	Dir,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct File {
	pub path: String,
	pub kind: FileType,
}

impl File {
	pub fn new(path: String, kind: FileType) -> Result<File> { Ok(File { path, kind }) }

	pub fn link(path: String, link: String) -> Result<File> {
		// Symlink will be the real file relative
		// I think..
		Self::new(path, FileType::Symlink(link))
	}

	pub fn from_slice(path: String, data: &[u8]) -> Result<File> {
		File::new(path, FileType::File(format!("{:x}", Sha256::digest(data))))
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DebFile {
	pub path: String,
	pub map: HashMap<String, Vec<File>>,
	pub control: Vec<HashMap<String, String>>,
	pub hash: String,
}

impl ProgressItem for DebFile {
	fn header(&self) -> String { "Hashing: ".to_string() }

	fn msg(&self) -> String { self.name().to_string() }
}

impl DebFile {
	pub async fn new(path: String) -> Result<DebFile> {
		let data = tokio::fs::read(&path).await?;
		let mut ar = Archive::new(data.as_slice());
		let hash = format!("{:x}", Sha256::digest(&data));

		let mut map: HashMap<String, Vec<File>> = HashMap::new();
		let mut control: Vec<HashMap<String, String>> = vec![];
		while let Some(res) = ar.next_entry() {
			let mut entry = res?;
			let tarball = std::str::from_utf8(entry.header().identifier())?.to_string();
			if !tarball.contains(".tar") {
				continue;
			}

			let mut tar = Tarchive::new(Cursor::new(entry.read_vec()?.decompress().await?));
			for file in tar.entries()? {
				let mut entry = file?;
				let path_str = entry.path()?.to_string_lossy().to_string();

				// Skip directories
				if path_str.ends_with("/") {
					continue;
				}
				if path_str == "./control" {
					control.push(
						tagfile::parse_tagfile(&entry.read_string()?)?
							.into_iter()
							.next()
							.unwrap()
							.into(),
					);
				}

				let string = entry.path()?.display().to_string();
				let file = match entry
					.header()
					.link_name()?
					.iter()
					.filter_map(|l| l.to_str())
					.next()
				{
					Some(link) => File::link(string, link.into()),
					None => File::from_slice(string, &entry.read_vec()?),
				}?;

				if !map.contains_key(&tarball) {
					map.insert(tarball.to_string(), vec![]);
				}
				map.get_mut(&tarball).unwrap().push(file);
			}
		}

		Ok(DebFile {
			path,
			map,
			control,
			hash,
		})
	}

	// pub fn path(&self) -> &Path { Path::new(&self.path) }

	pub fn name(&self) -> &str { self.control[0].get("Package").unwrap() }

	pub fn version(&self) -> &str { self.control[0].get("Version").unwrap() }

	pub fn to_json(&self) -> Result<String> { Ok(serde_json::to_string(self)?) }

	// TODO: What is the best way to know the file name
	// Right now it is based on the hash of the .deb
	pub async fn store(&self) -> Result<()> {
		Ok(format!("./test/{}", self.hash)
			.open()
			.await?
			.write_all(self.to_json()?.as_bytes())
			.await?)
	}
}
