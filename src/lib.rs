#![warn(missing_docs)]
// #![forbid(unsafe_code)]
#![no_std]

//! Safe Rust port of the LZO1X compression algorithm.
//!
//! # Examples
//!
//! Simple compression and decompression.
//!
//! ```
//! let data = &[0xaa; 100];
//! let compressed = lzo1x::compress(data, lzo1x::CompressLevel::default());
//!
//! assert_eq!(compressed.len(), 34);
//!
//! let mut decompressed = vec![0; data.len()];
//! lzo1x::decompress(&compressed, &mut decompressed).unwrap();
//!
//! assert_eq!(decompressed, data);
//! ```
//!
//! Slow pre-compression of data.
//!
//! ```
//! let data = &[0xaa; 100];
//! let mut compressed = lzo1x::compress(data, lzo1x::CompressLevel::new(13).unwrap());
//!
//! assert_eq!(compressed.len(), 9);
//!
//! lzo1x::optimize(&mut compressed, data.len());
//!
//! let mut decompressed = vec![0; data.len()];
//! lzo1x::decompress(&compressed, &mut decompressed).unwrap();
//!
//! assert_eq!(decompressed, data);
//! ```

extern crate alloc;

mod compress_1;
mod compress_999;
mod config;
mod decompress;
mod optimize;
mod swd;

pub use decompress::decompress;
pub use optimize::optimize;

use alloc::vec::Vec;

use compress_1::compress_1;
use compress_999::compress_999;

/// Compress the given `src` with the given compression `level`.
///
/// A higher compression level results in a better compression ratio, at the cost of a longer runtime.
///
/// # Examples
///
/// ```
/// let data = &[0xaa; 100];
/// let compressed = lzo1x::compress(data, lzo1x::CompressLevel::default());
///
/// assert_eq!(compressed.len(), 34);
/// ```
pub fn compress(src: &[u8], level: CompressLevel) -> Vec<u8> {
    match level.0 {
        1 => compress_1(src, 11),
        2 => compress_1(src, 12),
        3 => compress_1(src, 14),
        4 => compress_1(src, 15),
        5 => compress_999(src, 0, 0, 0, 8, 4, 0),
        6 => compress_999(src, 0, 0, 0, 16, 8, 0),
        7 => compress_999(src, 0, 0, 0, 32, 16, 0),
        8 => compress_999(src, 1, 5, 5, 16, 16, 0),
        9 => compress_999(src, 1, 8, 16, 32, 32, 0),
        10 => compress_999(src, 1, 8, 16, 128, 128, 0),
        11 => compress_999(src, 2, 8, 32, 128, 256, 0),
        12 => compress_999(src, 2, 32, 128, 2048, 2048, 1),
        13 => compress_999(src, 2, 2048, 2048, 2048, 4096, 1),
        _ => unreachable!(),
    }
}

/// LZO1X compression level.
#[derive(Clone, Copy)]
pub struct CompressLevel(u8);

impl CompressLevel {
    /// Create a new `CompressLevel` instance from the given `level`.
    ///
    /// The given `level` should be between 1 and 13, otherwise `None` is returned.
    ///
    /// The default compression level is 3.
    pub const fn new(level: u8) -> Option<Self> {
        if !matches!(level, 1..=13) {
            return None;
        }

        Some(Self(level))
    }
}

impl Default for CompressLevel {
    fn default() -> Self {
        Self(3)
    }
}

/// Error while decompressing using LZO1X.
#[derive(Debug)]
pub struct DecompressError;
