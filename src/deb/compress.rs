use anyhow::{bail, Result};
use async_compression::tokio::bufread::{GzipDecoder, XzDecoder, ZstdDecoder};
use tokio::io::AsyncRead;

use super::AsyncReader;

pub trait Decompress {
	async fn decompress(&self) -> Result<Vec<u8>>;
}

impl Decompress for Vec<u8> {
	async fn decompress(&self) -> Result<Vec<u8>> {
		Compression::compressor(self.as_slice())?.read_vec().await
	}
}

enum Compression {
	Ar,
	Tar,
	Gz,
	Xz,
	Zstd,
}

impl Compression {
	/// Matches the data with Known Magic Numbers and returns the compression
	/// type
	fn from_slice(slice: &[u8]) -> Result<Compression> {
		for magic in [Self::Ar, Self::Tar, Self::Gz, Self::Xz, Self::Zstd] {
			if slice.starts_with(magic.magic()) {
				return Ok(magic);
			}
		}
		bail!("Archive type is not supported");
	}

	/// Returns the compressor for this compressor type
	fn compressor(slice: &[u8]) -> Result<Box<dyn AsyncRead + std::marker::Unpin + Send + '_>> {
		Ok(match Compression::from_slice(slice)? {
			Self::Ar => todo!(),
			Self::Tar => todo!(),
			Self::Gz => Box::new(GzipDecoder::new(slice)),
			Self::Xz => Box::new(XzDecoder::new(slice)),
			Self::Zstd => Box::new(ZstdDecoder::new(slice)),
		})
	}

	/// Returns a slice to the Magic Number to identify compression
	fn magic(&self) -> &[u8] {
		match self {
			Self::Ar => &[0x21, 0x3C, 0x61, 0x72, 0x63, 0x68, 0x3E],
			Self::Tar => &[0x75, 0x73, 0x74, 0x61, 0x72],
			Self::Gz => &[0x1F, 0x8B],
			Self::Xz => &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00],
			Self::Zstd => &[0x28, 0xB5, 0x2F, 0xFD],
		}
	}
}
