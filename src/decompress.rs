use cfg_if::cfg_if;

use crate::DecompressError;

enum State {
    State0,
    State1,
    State2,
    State3,
    State4,
}

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

    let mut state: State;

    if src_len - src_idx < 1 {
        return Err(DecompressError);
    }

    if src[src_idx] > 17 {
        t = src[src_idx] as usize - 17;
        src_idx += 1;

        if t < 4 {
            state = State::State4;
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

            state = State::State1;
        }
    } else {
        state = State::State0;
    }

    loop {
        match state {
            State::State0 => {
                if src_len - src_idx < 3 {
                    return Err(DecompressError);
                }

                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = State::State2;
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

                    state = State::State1;
                }
            }
            State::State1 => {
                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = State::State2;
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

                    state = State::State3;
                }
            }
            State::State2 => {
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
                    t += 2;

                    copy_match(dst, m_pos, dst_idx, t)?;
                    dst_idx += t;

                    state = State::State3;
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
                    t += 2;

                    copy_match(dst, m_pos, dst_idx, t)?;
                    dst_idx += t;

                    state = State::State3;
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
                    t += 2;

                    copy_match(dst, m_pos, dst_idx, t)?;
                    dst_idx += t;

                    state = State::State3;
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

                    state = State::State3;
                }
            }
            State::State3 => {
                t = src[src_idx - 2] as usize & 3;

                if t == 0 {
                    state = State::State0;
                } else {
                    state = State::State4;
                }
            }
            State::State4 => {
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

                state = State::State2;
            }
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

fn copy_match(
    dst: &mut [u8],
    match_pos: usize,
    dst_idx: usize,
    len: usize,
) -> Result<(), DecompressError> {
    if match_pos >= dst_idx {
        return Err(DecompressError);
    }

    if dst.len() - dst_idx < len {
        return Err(DecompressError);
    }

    let match_off = dst_idx - match_pos;
    let dst = &mut dst[match_pos..match_pos + match_off + len];

    if match_off >= len {
        let (a, b) = dst.split_at_mut(match_off);
        b.copy_from_slice(&a[..b.len()]);
    } else if match_off == 1 {
        let value = dst[0];
        dst[match_off..].fill(value);
    } else if match_off <= 4 {
        let value: [u8; 4] = dst[..4].try_into().unwrap();
        let mut dst = &mut dst[match_off..];

        while dst.len() >= 4 {
            dst[..4].copy_from_slice(&value);
            dst = &mut dst[match_off..];
        }

        for i in 0..dst.len() {
            dst[i] = value[i % match_off];
        }
    } else if match_off <= 8 {
        let value: [u8; 8] = dst[..8].try_into().unwrap();
        let mut dst = &mut dst[match_off..];

        while dst.len() >= 8 {
            dst[..8].copy_from_slice(&value);
            dst = &mut dst[match_off..];
        }

        for i in 0..dst.len() {
            dst[i] = value[i % match_off];
        }
    } else {
        let mut dst = dst;

        loop {
            let (a, b) = dst.split_at_mut(match_off);

            if b.len() < 8 {
                break;
            }

            b[..8].copy_from_slice(&a[..8]);
            dst = &mut dst[8..];
        }

        let (a, b) = dst.split_at_mut(match_off);
        b.copy_from_slice(&a[..b.len()]);
    }

    Ok(())
}
