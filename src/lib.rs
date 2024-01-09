mod compress_1;
mod decompress;

pub use compress_1::compress_1;
pub use decompress::decompress;

#[derive(Debug)]
pub struct Error;
