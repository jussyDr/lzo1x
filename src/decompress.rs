/// Decompress the given `src` into the given `dst`.
///
/// #### Panics
///
/// Panics if the given `src` does not contain valid compressed data,
/// or if the given `dst` does not match the length of the decompressed data.
///
/// # Examples
///
/// ```
/// let data = &[0xaa; 100];
/// let compressed = lzo1x::compress(data, 3).unwrap();
///
/// let mut decompressed = vec![0; data.len()];
/// lzo1x::decompress(&compressed, &mut decompressed);
///
/// assert_eq!(decompressed, data);
/// ```
pub fn decompress(src: &[u8], dst: &mut [u8]) {
    let mut src_idx = 0;
    let mut dst_idx = 0;

    let mut t = 0;
    let mut m_pos = 0;

    let mut state: u8;

    if src[src_idx] > 17 {
        t = src[src_idx] as usize - 17;
        src_idx += 1;

        if t < 4 {
            state = 5;
        } else {
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
                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = 2;
                } else {
                    if t == 0 {
                        while src[src_idx] == 0 {
                            t += 255;
                            src_idx += 1;
                        }

                        t += 15 + src[src_idx] as usize;
                        src_idx += 1;
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
                    m_pos = dst_idx - (1 + 0x0800);
                    m_pos -= t >> 2;
                    m_pos -= (src[src_idx] as usize) << 2;
                    src_idx += 1;

                    for i in 0..3 {
                        dst[dst_idx + i] = dst[m_pos + i];
                    }

                    dst_idx += 3;
                    m_pos += 3;

                    state = 4;
                }
            }
            2 => {
                if t >= 64 {
                    m_pos = dst_idx - 1;
                    m_pos -= (t >> 2) & 7;
                    m_pos -= (src[src_idx] as usize) << 3;
                    src_idx += 1;
                    t = (t >> 5) - 1;

                    state = 3;
                } else if t >= 32 {
                    t &= 31;

                    if t == 0 {
                        while src[src_idx] == 0 {
                            t += 255;
                            src_idx += 1;
                        }

                        t += 31 + src[src_idx] as usize;
                        src_idx += 1;
                    }

                    m_pos = dst_idx - 1;
                    m_pos -= (src[src_idx] as usize >> 2) + ((src[src_idx + 1] as usize) << 6);
                    src_idx += 2;

                    state = 3;
                } else if t >= 16 {
                    m_pos = dst_idx;
                    m_pos -= (t & 8) << 11;

                    t &= 7;

                    if t == 0 {
                        while src[src_idx] == 0 {
                            t += 255;
                            src_idx += 1;
                        }

                        t += 7 + src[src_idx] as usize;
                        src_idx += 1;
                    }

                    m_pos -= (src[src_idx] as usize >> 2) + ((src[src_idx + 1] as usize) << 6);
                    src_idx += 2;

                    if m_pos == dst_idx {
                        break;
                    }

                    m_pos -= 0x4000;

                    state = 3;
                } else {
                    m_pos = dst_idx - 1;
                    m_pos -= t >> 2;
                    m_pos -= (src[src_idx] as usize) << 2;
                    src_idx += 1;

                    for i in 0..2 {
                        dst[dst_idx + i] = dst[m_pos + i];
                    }

                    dst_idx += 2;
                    m_pos += 2;

                    state = 4;
                }
            }
            3 => {
                for i in 0..t + 2 {
                    dst[dst_idx + i] = dst[m_pos + i];
                }

                dst_idx += t + 2;
                m_pos += t + 2;
                t = 0;

                state = 4;
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

    if src_idx < src.len() {
        panic!();
    }

    if dst_idx < dst.len() {
        panic!();
    }
}
