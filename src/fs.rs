use std::fmt::Debug;
use std::path::Path;

use anyhow::{Context, Result};
use tokio::fs;
use tokio::io::BufWriter;

// Generate functions with the same names as std::fs counterparts
// fs_with_context!(read, Vec<u8>);
// fs_with_context!(write, [u8], false, ());

macro_rules! fs_method {
	($name:ident, { $func:path => $ret:ty }) => {
		async fn $name(&self) -> Result<$ret> {
			$func(self).await.with_context(|| format!(
				"Failed to {} {self:?}",
				stringify!($name)
			))
		}
	};

	// Two options the return value is None
	($name:ident, { $func:path }) => {
		fs_method!($name, { $func => () });
	};

	($name:ident, { $func:path, with_arg }) => {
		fs_method!($name, { $func => (), with_arg });
	};

	($name:ident, { $func:path => $ret:ty, with_arg }) => {
		async fn $name<T: AsyncPath>(&self, other: T) -> Result<$ret> {
			$func(self, &other).await.with_context(|| format!(
				"Failed to {} {self:?} => {other:?}",
				stringify!($name)
			))
		}
	};
}

macro_rules! async_fs {
	($( $name:ident $args:tt ),*) => {
		pub trait AsyncPath: AsRef<Path> + Debug {}
		impl<P: AsRef<Path> + Debug> AsyncPath for P {}

		pub trait AsyncFs {
			async fn open(&self) -> Result<fs::File>;
			async fn open_writer(&self) -> Result<BufWriter<fs::File>>;

			async fn read_string(&self) -> Result<String>;
			async fn remove(&self) -> Result<()>;
			async fn remove_recurse(&self) -> Result<()>;
			async fn mkdir(&self) -> Result<()>;

			async fn cp<T: AsyncPath>(&self, other: T) -> Result<u64>;
			async fn rename<T: AsyncPath>(&self, other: T) -> Result<()>;
		}

		impl<P: AsyncPath> AsyncFs for P {
			$(fs_method!($name, $args);)*

			async fn open_writer(&self) -> Result<BufWriter<fs::File>> {
				Ok(BufWriter::new(self.open().await?))
			}
		}

	}
}

async_fs!(
	open { fs::File::create => fs::File },
	read_string { fs::read_to_string => String },
	remove { fs::remove_file },
	remove_recurse { fs::remove_dir_all },
	mkdir { fs::create_dir_all },
	cp { fs::copy => u64, with_arg },
	rename { fs::rename, with_arg }
);
