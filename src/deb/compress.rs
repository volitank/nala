use anyhow::{bail, Result};
use async_compression::tokio::bufread::{GzipDecoder, XzDecoder, ZstdDecoder};
use tokio::io::AsyncRead;

use super::AsyncReader;

pub trait Decompress {
	async fn decompress(&self) -> Result<Vec<u8>>;
}

impl Decompress for Vec<u8> {
	async fn decompress(&self) -> Result<Vec<u8>> {
		Compression::from_slice(self.as_slice())?
			.decompressor(self.as_slice())?
			.read_vec()
			.await
	}
}

enum Compression {
	Gz,
	Xz,
	Zstd,
}

impl Compression {
	/// Matches the data with Known Magic Numbers and returns the compression
	/// type
	fn from_slice(slice: &[u8]) -> Result<Compression> {
		for compression in [Self::Gz, Self::Xz, Self::Zstd] {
			if slice.starts_with(compression.magic()) {
				return Ok(compression);
			}
		}
		bail!("Archive type is not supported");
	}

	/// Returns the decompressor for this compression type.
	fn decompressor<'a>(
		&self,
		slice: &'a [u8],
	) -> Result<Box<dyn AsyncRead + std::marker::Unpin + Send + 'a>> {
		Ok(match self {
			Self::Gz => Box::new(GzipDecoder::new(slice)),
			Self::Xz => Box::new(XzDecoder::new(slice)),
			Self::Zstd => Box::new(ZstdDecoder::new(slice)),
		})
	}

	/// Returns a slice to the Magic Number to identify compression
	fn magic(&self) -> &[u8] {
		match self {
			Self::Gz => &[0x1F, 0x8B],
			Self::Xz => &[0xFD, 0x37, 0x7A, 0x58, 0x5A, 0x00],
			Self::Zstd => &[0x28, 0xB5, 0x2F, 0xFD],
		}
	}
}

#[cfg(test)]
mod tests {
	use async_compression::tokio::write::GzipEncoder;
	use tokio::io::AsyncWriteExt;

	use super::*;

	#[tokio::test]
	async fn gzip_data_decompresses() {
		let data = b"Package: demo\nVersion: 1.0\n";
		let mut encoder = GzipEncoder::new(Vec::new());
		encoder.write_all(data).await.unwrap();
		encoder.shutdown().await.unwrap();
		let compressed = encoder.into_inner();

		assert_eq!(compressed.decompress().await.unwrap(), data);
	}

	#[tokio::test]
	async fn raw_tar_is_not_decompressed_here() {
		let data = b"not compressed".to_vec();

		let err = data.decompress().await.unwrap_err();

		assert!(err.to_string().contains("Archive type is not supported"));
	}
}
