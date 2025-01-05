use std::io::Read;

use anyhow::Result;
use tokio::io::{AsyncRead, AsyncReadExt};

pub trait Reader {
	fn read_vec(&mut self) -> Result<Vec<u8>>;

	fn read_string(&mut self) -> Result<String>;
}

pub trait AsyncReader {
	async fn read_vec(&mut self) -> Result<Vec<u8>>;
}

impl<T: Read> Reader for T {
	fn read_vec(&mut self) -> Result<Vec<u8>> {
		let mut buf = vec![];
		self.read_to_end(&mut buf)?;
		Ok(buf)
	}

	fn read_string(&mut self) -> Result<String> {
		let mut buf = String::new();
		self.read_to_string(&mut buf)?;
		Ok(buf)
	}
}

impl<T: AsyncRead + std::marker::Unpin> AsyncReader for T {
	async fn read_vec(&mut self) -> Result<Vec<u8>> {
		let mut buf = vec![];
		self.read_to_end(&mut buf).await?;
		Ok(buf)
	}
}
