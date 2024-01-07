mod compress_1;
mod decompress;

pub use compress_1::{compress_1, compress_1_11, compress_1_12, compress_1_15};
pub use decompress::decompress;

#[derive(Debug)]
pub struct Error;
