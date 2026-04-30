use std::collections::HashMap;
use std::io::Cursor;

use anyhow::{anyhow, Result};
use ar::Archive;
use rust_apt::tagfile;
use serde::{Deserialize, Serialize};
use tar::Archive as Tarchive;
use tokio::fs;

use super::{Decompress, Reader};
use crate::hashsum;

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
		let hash = hashsum::sha256_hex(&data);

		let mut control: Option<HashMap<String, String>> = None;
		while let Some(res) = ar.next_entry() {
			let mut entry = res?;
			let tarball = std::str::from_utf8(entry.header().identifier())?;
			if !tarball.starts_with("control.tar") {
				continue;
			}

			let data = if tarball == "control.tar" {
				entry.read_vec()?
			} else {
				entry.read_vec()?.decompress().await?
			};

			let mut tar = Tarchive::new(Cursor::new(data));
			for file in tar.entries()? {
				let mut entry = file?;
				let path = entry.path()?;
				if path.as_os_str() != "./control" && path.as_os_str() != "control" {
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

#[cfg(test)]
mod tests {
	use std::io::Cursor;
	use std::path::PathBuf;
	use std::time::{SystemTime, UNIX_EPOCH};

	use async_compression::tokio::write::{GzipEncoder, XzEncoder, ZstdEncoder};
	use tokio::io::AsyncWriteExt;

	use super::*;

	enum ControlArchive {
		Raw,
		Gzip,
		Xz,
		Zstd,
	}

	impl ControlArchive {
		fn filename(&self) -> &'static [u8] {
			match self {
				Self::Raw => b"control.tar",
				Self::Gzip => b"control.tar.gz",
				Self::Xz => b"control.tar.xz",
				Self::Zstd => b"control.tar.zst",
			}
		}

		fn label(&self) -> &'static str {
			match self {
				Self::Raw => "raw",
				Self::Gzip => "gz",
				Self::Xz => "xz",
				Self::Zstd => "zst",
			}
		}
	}

	fn control_tar() -> Vec<u8> {
		let control = b"Package: demo\nVersion: 1.0\nArchitecture: all\n";
		let mut data = Vec::new();
		{
			let mut builder = tar::Builder::new(&mut data);
			let mut header = tar::Header::new_gnu();
			header.set_size(control.len() as u64);
			header.set_mode(0o644);
			header.set_cksum();
			builder
				.append_data(&mut header, "./control", control.as_slice())
				.unwrap();
			builder.finish().unwrap();
		}
		data
	}

	async fn compress_control(kind: &ControlArchive, data: &[u8]) -> Vec<u8> {
		match kind {
			ControlArchive::Raw => data.to_vec(),
			ControlArchive::Gzip => {
				let mut encoder = GzipEncoder::new(Vec::new());
				encoder.write_all(data).await.unwrap();
				encoder.shutdown().await.unwrap();
				encoder.into_inner()
			},
			ControlArchive::Xz => {
				let mut encoder = XzEncoder::new(Vec::new());
				encoder.write_all(data).await.unwrap();
				encoder.shutdown().await.unwrap();
				encoder.into_inner()
			},
			ControlArchive::Zstd => {
				let mut encoder = ZstdEncoder::new(Vec::new());
				encoder.write_all(data).await.unwrap();
				encoder.shutdown().await.unwrap();
				encoder.into_inner()
			},
		}
	}

	async fn write_deb_fixture(kind: ControlArchive) -> PathBuf {
		let control = compress_control(&kind, &control_tar()).await;
		let mut builder = ar::Builder::new(Vec::new());
		for (name, data) in [
			(b"debian-binary".as_slice(), b"2.0\n".as_slice()),
			(kind.filename(), control.as_slice()),
		] {
			let header = ar::Header::new(name.to_vec(), data.len() as u64);
			builder.append(&header, Cursor::new(data)).unwrap();
		}
		let deb = builder.into_inner().unwrap();

		let id = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap()
			.as_nanos();
		let path = std::env::temp_dir().join(format!("nala-debfile-{}-{id}.deb", kind.label()));
		fs::write(&path, deb).await.unwrap();
		path
	}

	#[tokio::test]
	async fn debfile_reads_raw_control_tar() {
		let path = write_deb_fixture(ControlArchive::Raw).await;

		let deb = DebFile::new(path.to_string_lossy().into_owned())
			.await
			.unwrap();
		fs::remove_file(&path).await.unwrap();

		assert_eq!(deb.name(), "demo");
		assert_eq!(deb.version(), "1.0");
	}

	#[tokio::test]
	async fn debfile_reads_gzip_control_tar() {
		let path = write_deb_fixture(ControlArchive::Gzip).await;

		let deb = DebFile::new(path.to_string_lossy().into_owned())
			.await
			.unwrap();
		fs::remove_file(&path).await.unwrap();

		assert_eq!(deb.name(), "demo");
		assert_eq!(deb.version(), "1.0");
	}

	#[tokio::test]
	async fn debfile_reads_xz_control_tar() {
		let path = write_deb_fixture(ControlArchive::Xz).await;

		let deb = DebFile::new(path.to_string_lossy().into_owned())
			.await
			.unwrap();
		fs::remove_file(&path).await.unwrap();

		assert_eq!(deb.name(), "demo");
		assert_eq!(deb.version(), "1.0");
	}

	#[tokio::test]
	async fn debfile_reads_zstd_control_tar() {
		let path = write_deb_fixture(ControlArchive::Zstd).await;

		let deb = DebFile::new(path.to_string_lossy().into_owned())
			.await
			.unwrap();
		fs::remove_file(&path).await.unwrap();

		assert_eq!(deb.name(), "demo");
		assert_eq!(deb.version(), "1.0");
	}
}
