// Decompression is based on the following description: https://docs.kernel.org/staging/lzo.html.

use crate::DecompressError;

/// Decompress the given `src` into the given `dst`.
///
/// #### Errors
///
/// This function returns an error if the given `src` does not contain valid compressed data,
/// or if the given `dst` does not exactly match the length of the decompressed data.
///
/// # Examples
///
/// ```
/// let data = &[0xaa; 100];
/// let compressed = lzo1x::compress(data, lzo1x::CompressLevel::default());
///
/// let mut decompressed = vec![0; data.len()];
/// lzo1x::decompress(&compressed, &mut decompressed).unwrap();
///
/// assert_eq!(decompressed, data);
/// ```
pub fn decompress(src: &[u8], dst: &mut [u8]) -> Result<(), DecompressError> {
    enum State {
        A,
        B,
        C,
    }

    let mut src_pos = 0;
    let mut dst_pos = 0;

    if src_pos + 1 > src.len() {
        return Err(DecompressError::InvalidInput);
    }

    let insn = src[src_pos];

    let mut state = if insn <= 17 {
        State::A
    } else {
        src_pos += 1;

        let lit_len = (insn as usize) - 17;

        // Copy literal with length in the range 0-238.

        if src_pos + lit_len > src.len() {
            return Err(DecompressError::InvalidInput);
        }

        if dst_pos + lit_len > dst.len() {
            return Err(DecompressError::OutputLength);
        }

        for _ in 0..lit_len {
            dst[dst_pos] = src[src_pos];
            src_pos += 1;
            dst_pos += 1;
        }

        if insn <= 20 { State::B } else { State::C }
    };

    loop {
        // Decode instruction.

        if src_pos + 1 > src.len() {
            return Err(DecompressError::InvalidInput);
        }

        let insn = src[src_pos];
        src_pos += 1;

        let (match_len, match_dist, lit_insn) = match insn {
            0..=15 => {
                let (match_len, match_dist_offset) = match state {
                    State::A => {
                        let lit_len = if insn == 0 {
                            let start_src_pos = src_pos;

                            loop {
                                if src_pos + 1 > src.len() {
                                    return Err(DecompressError::InvalidInput);
                                }

                                if src[src_pos] != 0 {
                                    break;
                                }

                                src_pos += 1;
                            }

                            let count = src_pos - start_src_pos;

                            let lit_len = (count * 255) + (src[src_pos] as usize) + 18;
                            src_pos += 1;

                            lit_len
                        } else {
                            (insn as usize) + 3
                        };

                        // Copy literal with length 4 or greater.

                        if src_pos + lit_len > src.len() {
                            return Err(DecompressError::InvalidInput);
                        }

                        if dst_pos + lit_len > dst.len() {
                            return Err(DecompressError::OutputLength);
                        }

                        dst[dst_pos..dst_pos + lit_len]
                            .copy_from_slice(&src[src_pos..src_pos + lit_len]);
                        src_pos += lit_len;
                        dst_pos += lit_len;

                        state = State::C;

                        continue;
                    }
                    State::B => (2, 1),
                    State::C => (3, 2049),
                };

                if src_pos + 1 > src.len() {
                    return Err(DecompressError::InvalidInput);
                }

                let match_dist =
                    ((src[src_pos] as usize) << 2) + ((insn >> 2) as usize) + match_dist_offset;
                src_pos += 1;

                (match_len, match_dist, insn)
            }
            16..=31 => {
                let match_len = if (insn & 0b00000111) == 0 {
                    let src_pos_start = src_pos;

                    loop {
                        if src_pos + 1 > src.len() {
                            return Err(DecompressError::InvalidInput);
                        }

                        if src[src_pos] != 0 {
                            break;
                        }

                        src_pos += 1;
                    }

                    let count = src_pos - src_pos_start;

                    let match_len = (count * 255) + (src[src_pos] as usize) + 9;
                    src_pos += 1;

                    match_len
                } else {
                    ((insn & 0b00000111) as usize) + 2
                };

                if src_pos + 2 > src.len() {
                    return Err(DecompressError::InvalidInput);
                }

                let match_dist = ((((insn & 0b00001000) >> 3) as usize) << 14)
                    + ((src[src_pos + 1] as usize) << 6)
                    + ((src[src_pos] >> 2) as usize)
                    + 16384;
                let sub_insn = src[src_pos];
                src_pos += 2;

                if match_dist == 16384 {
                    break;
                }

                (match_len, match_dist, sub_insn)
            }
            32..=63 => {
                let match_len = if (insn & 0b00011111) == 0 {
                    let src_pos_start = src_pos;

                    loop {
                        if src_pos + 1 > src.len() {
                            return Err(DecompressError::InvalidInput);
                        }

                        if src[src_pos] != 0 {
                            break;
                        }

                        src_pos += 1;
                    }

                    let count = src_pos - src_pos_start;

                    let match_len = (count * 255) + (src[src_pos] as usize) + 33;
                    src_pos += 1;

                    match_len
                } else {
                    ((insn & 0b00011111) as usize) + 2
                };

                if src_pos + 2 > src.len() {
                    return Err(DecompressError::InvalidInput);
                }

                let match_dist =
                    ((src[src_pos + 1] as usize) << 6) + ((src[src_pos] >> 2) as usize) + 1;
                let sub_insn = src[src_pos];
                src_pos += 2;

                (match_len, match_dist, sub_insn)
            }
            64..=255 => {
                let match_len = match insn {
                    0..=63 => unreachable!(),
                    64..=127 => {
                        if insn & 0b00100000 != 0 {
                            4
                        } else {
                            3
                        }
                    }
                    128..=255 => (((insn & 0b01100000) >> 5) as usize) + 5,
                };

                if src_pos + 1 > src.len() {
                    return Err(DecompressError::InvalidInput);
                }

                let match_dist =
                    ((src[src_pos] as usize) << 3) + (((insn & 0b00011100) >> 2) as usize) + 1;
                src_pos += 1;

                (match_len, match_dist, insn)
            }
        };

        // Copy match.

        if match_dist > dst_pos {
            return Err(DecompressError::InvalidInput);
        }

        if dst_pos + match_len > dst.len() {
            return Err(DecompressError::OutputLength);
        }

        let match_pos = dst_pos - match_dist;

        if match_dist >= match_len {
            // Match does not overlap.

            let (a, b) = dst.split_at_mut(dst_pos);
            b[..match_len].copy_from_slice(&a[match_pos..match_pos + match_len]);
        } else {
            // Match overlaps.

            let (a, b) = dst.split_at_mut(dst_pos);
            b[..match_dist].copy_from_slice(&a[match_pos..match_pos + match_dist]);

            let mut n = match_dist;

            while n * 2 < match_len {
                let (a, b) = b.split_at_mut(n);
                b[..n].copy_from_slice(a);

                n *= 2;
            }

            let (a, b) = b.split_at_mut(n);
            b[..match_len - n].copy_from_slice(&a[..match_len - n]);
        }

        dst_pos += match_len;

        let lit_len = (lit_insn & 0b00000011) as usize;

        state = if lit_len == 0 {
            State::A
        } else {
            // Copy literal with length in the range 1-3.

            if src_pos + lit_len > src.len() {
                return Err(DecompressError::InvalidInput);
            }

            if dst_pos + lit_len > dst.len() {
                return Err(DecompressError::OutputLength);
            }

            let src = &src[src_pos..src_pos + lit_len];
            let dst = &mut dst[dst_pos..dst_pos + lit_len];

            for (src, dst) in src.iter().zip(dst).take(3) {
                *dst = *src;
            }

            src_pos += lit_len;
            dst_pos += lit_len;

            State::B
        };
    }

    // Ensure the source buffer was completely consumed.
    if src_pos != src.len() {
        return Err(DecompressError::InvalidInput);
    }

    // Ensure the destination buffer was completely filled.
    if dst_pos != dst.len() {
        return Err(DecompressError::OutputLength);
    }

    Ok(())
}
