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

    let mut state: u8;

    if src_len - src_idx < 1 {
        return Err(DecompressError);
    }

    if src[src_idx] > 17 {
        t = src[src_idx] as usize - 17;
        src_idx += 1;

        if t < 4 {
            state = 5;
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

                    state = 1;
                }
            }
            1 => {
                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = 2;
                } else {
                    let mut m_pos = dst_idx;

                    let mut x = 1 + 0x0800;
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

                    state = 4;
                }
            }
            2 => {
                if t >= 64 {
                    let mut m_pos = dst_idx;

                    let mut x = 1;
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

                    t += 2;

                    if dst_len - dst_idx < t {
                        return Err(DecompressError);
                    }

                    assert!(m_pos < dst_idx); // helps eliminate bound checks in next loop

                    for i in 0..t {
                        dst[dst_idx + i] = dst[m_pos + i];
                    }

                    dst_idx += t;

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

                    let mut m_pos = dst_idx;

                    let mut x = 1;

                    cfg_if! {
                        if #[cfg(target_endian = "little")] {
                            x += u16::from_le_bytes(src[src_idx..src_idx + 2].try_into().unwrap()) as usize
                            >> 2;
                        } else {
                            x += (src[src_idx] as usize >> 2) + ((src[src_idx + 1] as usize) << 6);
                        }
                    }

                    src_idx += 2;

                    if m_pos < x {
                        return Err(DecompressError);
                    }

                    m_pos -= x;

                    if m_pos >= dst_idx {
                        return Err(DecompressError);
                    }

                    t += 2;

                    if dst_len - dst_idx < t {
                        return Err(DecompressError);
                    }

                    assert!(m_pos < dst_idx); // helps eliminate bound checks in next loop

                    if dst_idx - m_pos >= 8 {
                        while t >= 8 {
                            unsafe {
                                *(dst.as_mut_ptr().add(dst_idx) as *mut u64) =
                                    *(dst.as_ptr().add(m_pos) as *const u64);
                            }

                            dst_idx += 8;
                            m_pos += 8;
                            t -= 8;
                        }

                        if t >= 4 {
                            unsafe {
                                *(dst.as_mut_ptr().add(dst_idx) as *mut u32) =
                                    *(dst.as_ptr().add(m_pos) as *const u32);
                            }

                            dst_idx += 4;
                            m_pos += 4;
                            t -= 4;
                        }

                        if t > 0 {
                            dst[dst_idx] = dst[m_pos];
                            dst_idx += 1;

                            if t > 1 {
                                dst[dst_idx] = dst[m_pos + 1];
                                dst_idx += 1;

                                if t > 2 {
                                    dst[dst_idx] = dst[m_pos + 2];
                                    dst_idx += 1;
                                }
                            }
                        }
                    } else if dst_idx - m_pos >= 4 {
                        while t >= 4 {
                            unsafe {
                                *(dst.as_mut_ptr().add(dst_idx) as *mut u32) =
                                    *(dst.as_ptr().add(m_pos) as *const u32);
                            }

                            dst_idx += 4;
                            m_pos += 4;
                            t -= 4;
                        }

                        if t > 0 {
                            dst[dst_idx] = dst[m_pos];
                            dst_idx += 1;

                            if t > 1 {
                                dst[dst_idx] = dst[m_pos + 1];
                                dst_idx += 1;

                                if t > 2 {
                                    dst[dst_idx] = dst[m_pos + 2];
                                    dst_idx += 1;
                                }
                            }
                        }
                    } else {
                        for i in 0..t {
                            dst[dst_idx + i] = dst[m_pos + i];
                        }

                        dst_idx += t;
                    }

                    state = 4;
                } else if t >= 16 {
                    let mut m_pos = dst_idx;

                    let mut x = (t & 8) << 11;

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

                    if m_pos < x {
                        return Err(DecompressError);
                    }

                    m_pos -= x;

                    if m_pos >= dst_idx {
                        return Err(DecompressError);
                    }

                    t += 2;

                    if dst_len - dst_idx < t {
                        return Err(DecompressError);
                    }

                    assert!(m_pos < dst_idx); // helps eliminate bound checks in next loop

                    for i in 0..t {
                        dst[dst_idx + i] = dst[m_pos + i];
                    }

                    dst_idx += t;

                    state = 4;
                } else {
                    let mut m_pos = dst_idx;

                    let mut x = 1;
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

                    state = 4;
                }
            }
            4 => {
                t = src[src_idx - 2] as usize & 3;

                if t == 0 {
                    state = 0;
                } else {
                    state = 5;
                }
            }
            5 => {
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
