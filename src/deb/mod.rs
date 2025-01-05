mod compress;
mod debfile;
mod reader;

pub use compress::Decompress;
pub use debfile::DebFile;
pub use reader::{AsyncReader, Reader};
