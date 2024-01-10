mod compress_1;
mod compress_999;
mod config;
mod decompress;
mod swd;

pub use compress_1::compress_1;
pub use compress_999::compress_999;
pub use decompress::decompress;

#[derive(Debug)]
pub struct Error;
