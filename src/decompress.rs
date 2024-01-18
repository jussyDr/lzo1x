use cfg_if::cfg_if;

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
    let src_len = src.len();
    let dst_len = dst.len();

    let mut src_idx = 0;
    let mut dst_idx = 0;

    let mut t = 0;
    let mut m_pos = 0;
    let mut x = 0;

    let mut state: u8;

    if src_len - src_idx < 1 {
        return Err(DecompressError);
    }

    if src[src_idx] > 17 {
        t = src[src_idx] as usize - 17;
        src_idx += 1;

        if t < 4 {
            state = 6;
        } else {
            if dst_len - dst_idx < t {
                return Err(DecompressError);
            }

            if src_len - src_idx < t + 3 {
                return Err(DecompressError);
            }

            dst[dst_idx..dst_idx + t].copy_from_slice(&src[src_idx..src_idx + t]);
            dst_idx += t;
            src_idx += t;
            t = 0;

            state = 1;
        }
    } else {
        state = 0;
    }

    loop {
        match state {
            0 => {
                if src_len - src_idx < 3 {
                    return Err(DecompressError);
                }

                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = 2;
                } else {
                    if t == 0 {
                        while src[src_idx] == 0 {
                            t += 255;
                            src_idx += 1;

                            if t > usize::MAX - 510 {
                                return Err(DecompressError);
                            }

                            if src_len - src_idx < 1 {
                                return Err(DecompressError);
                            }
                        }

                        t += 15 + src[src_idx] as usize;
                        src_idx += 1;
                    }

                    if dst_len - dst_idx < t + 3 {
                        return Err(DecompressError);
                    }

                    if src_len - src_idx < t + 6 {
                        return Err(DecompressError);
                    }

                    dst[dst_idx..dst_idx + t + 3].copy_from_slice(&src[src_idx..src_idx + t + 3]);
                    dst_idx += t + 3;
                    src_idx += t + 3;
                    t = 0;

                    state = 1;
                }
            }
            1 => {
                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = 2;
                } else {
                    m_pos = dst_idx;

                    x = 1 + 0x0800;
                    x += t >> 2;
                    x += (src[src_idx] as usize) << 2;
                    src_idx += 1;

                    if m_pos < x {
                        return Err(DecompressError);
                    }

                    m_pos -= x;

                    if m_pos >= dst_idx {
                        return Err(DecompressError);
                    }

                    if dst_len - dst_idx < 3 {
                        return Err(DecompressError);
                    }

                    for i in 0..3 {
                        dst[dst_idx + i] = dst[m_pos + i];
                    }

                    dst_idx += 3;
                    m_pos += 3;

                    state = 5;
                }
            }
            2 => {
                if t >= 64 {
                    m_pos = dst_idx;

                    x = 1;
                    x += (t >> 2) & 7;
                    x += (src[src_idx] as usize) << 3;

                    src_idx += 1;
                    t = (t >> 5) - 1;

                    if m_pos < x {
                        return Err(DecompressError);
                    }

                    m_pos -= x;

                    if m_pos >= dst_idx {
                        return Err(DecompressError);
                    }

                    if dst_len - dst_idx < t + 2 {
                        return Err(DecompressError);
                    }

                    state = 4;
                } else if t >= 32 {
                    t &= 31;

                    if t == 0 {
                        while src[src_idx] == 0 {
                            t += 255;
                            src_idx += 1;

                            if t > usize::MAX - 510 {
                                return Err(DecompressError);
                            }

                            if src_len - src_idx < 1 {
                                return Err(DecompressError);
                            }
                        }

                        t += 31 + src[src_idx] as usize;
                        src_idx += 1;

                        if src_len - src_idx < 2 {
                            return Err(DecompressError);
                        }
                    }

                    m_pos = dst_idx;

                    x = 1;

                    cfg_if! {
                        if #[cfg(target_endian = "little")] {
                            x += u16::from_le_bytes(src[src_idx..src_idx + 2].try_into().unwrap()) as usize
                            >> 2;
                        } else {
                            x += (src[src_idx] as usize >> 2) + ((src[src_idx + 1] as usize) << 6);
                        }
                    }

                    src_idx += 2;

                    state = 3;
                } else if t >= 16 {
                    m_pos = dst_idx;

                    x = (t & 8) << 11;

                    t &= 7;

                    if t == 0 {
                        while src[src_idx] == 0 {
                            t += 255;
                            src_idx += 1;

                            if t > usize::MAX - 510 {
                                return Err(DecompressError);
                            }

                            if src_len - src_idx < 1 {
                                return Err(DecompressError);
                            }
                        }

                        t += 7 + src[src_idx] as usize;
                        src_idx += 1;

                        if src_len - src_idx < 2 {
                            return Err(DecompressError);
                        }
                    }

                    x += (src[src_idx] as usize >> 2) + ((src[src_idx + 1] as usize) << 6);
                    src_idx += 2;

                    if x == 0 {
                        break;
                    }

                    x += 0x4000;

                    state = 3;
                } else {
                    m_pos = dst_idx;

                    x = 1;
                    x += t >> 2;
                    x += (src[src_idx] as usize) << 2;
                    src_idx += 1;

                    if m_pos < x {
                        return Err(DecompressError);
                    }

                    m_pos -= x;

                    if m_pos >= dst_idx {
                        return Err(DecompressError);
                    }

                    if dst_len - dst_idx < 2 {
                        return Err(DecompressError);
                    }

                    for i in 0..2 {
                        dst[dst_idx + i] = dst[m_pos + i];
                    }

                    dst_idx += 2;
                    m_pos += 2;

                    state = 5;
                }
            }
            3 => {
                if m_pos < x {
                    return Err(DecompressError);
                }

                m_pos -= x;

                if m_pos >= dst_idx {
                    return Err(DecompressError);
                }

                if dst_len - dst_idx < t + 2 {
                    return Err(DecompressError);
                }

                state = 4;
            }
            4 => {
                assert!(m_pos < dst_idx); // helps with bound checks

                for i in 0..t + 2 {
                    dst[dst_idx + i] = dst[m_pos + i];
                }

                dst_idx += t + 2;
                m_pos += t + 2;
                t = 0;

                state = 5;
            }
            5 => {
                t = src[src_idx - 2] as usize & 3;

                if t == 0 {
                    state = 0;
                } else {
                    state = 6;
                }
            }
            6 => {
                if dst_len - dst_idx < t {
                    return Err(DecompressError);
                }

                if src_len - src_idx < t + 3 {
                    return Err(DecompressError);
                }

                dst[dst_idx] = src[src_idx];
                dst_idx += 1;
                src_idx += 1;

                if t > 1 {
                    dst[dst_idx] = src[src_idx];
                    dst_idx += 1;
                    src_idx += 1;

                    if t > 2 {
                        dst[dst_idx] = src[src_idx];
                        dst_idx += 1;
                        src_idx += 1;
                    }
                }

                t = src[src_idx] as usize;
                src_idx += 1;

                state = 2;
            }
            _ => unreachable!(),
        }
    }

    if src_idx < src_len {
        return Err(DecompressError);
    }

    if src_idx > src_len {
        return Err(DecompressError);
    }

    if dst_idx < dst_len {
        return Err(DecompressError);
    }

    Ok(())
}
