#[derive(Debug)]
pub struct Error;

pub fn decompress(src: &[u8], dst: &mut [u8]) -> Result<(), Error> {
    let mut src_idx = 0;
    let mut dst_idx = 0;

    let mut t = 0;
    let mut m_pos = 0;

    let mut state;

    if src[src_idx] > 17 {
        t = src[src_idx] as usize - 17;
        src_idx += 1;

        if t < 4 {
            state = 6;
        } else {
            state = 0;
        }
    } else {
        state = 1;
    }

    loop {
        match state {
            0 => {
                dst[dst_idx..dst_idx + t].copy_from_slice(&src[src_idx..src_idx + t]);
                dst_idx += t;
                src_idx += t;
                t = 0;

                state = 2;
            }
            1 => {
                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = 3;
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

                    state = 2;
                }
            }
            2 => {
                t = src[src_idx] as usize;
                src_idx += 1;

                if t >= 16 {
                    state = 3;
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

                    state = 5;
                }
            }
            3 => {
                if t >= 64 {
                    m_pos = dst_idx - 1;
                    m_pos -= (t >> 2) & 7;
                    m_pos -= (src[src_idx] as usize) << 3;
                    src_idx += 1;
                    t = (t >> 5) - 1;

                    state = 4;
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

                    state = 4;
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

                    state = 4;
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

                    state = 5;
                }
            }
            4 => {
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
                    state = 1;
                } else {
                    state = 6;
                }
            }
            6 => {
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

                state = 3;
            }
            _ => unreachable!(),
        }
    }

    if dst_idx != dst.len() {
        panic!()
    }

    Ok(())
}

pub fn compress(src: &[u8]) -> Vec<u8> {
    let src_len = src.len();

    let mut dst = vec![0; src_len + (src_len / 16) + 64 + 3];
    let mut work_mem = vec![0; 16384];

    let mut src_idx = 0;
    let mut dst_idx = 0;
    let mut l = src_len;
    let mut t = 0;

    while l > 20 {
        let ll = l.min(49152);

        if (t + ll) >> 5 == 0 {
            break;
        }

        work_mem.fill(0);

        let (new_t, out_len) = {
            let src_start = src_idx;
            let src_len = ll;
            let dst_start = dst_idx;
            let dict = &mut work_mem;

            let mut src_idx = src_start;
            let mut dst_idx = dst_idx;
            let mut ti = t;
            let mut ii = src_idx;

            if ti < 4 {
                src_idx += 4 - ti;
            }

            src_idx += 1 + ((src_idx - ii) >> 5);

            'main_loop: loop {
                let mut m_pos;

                loop {
                    if src_idx >= src_start + src_len - 20 {
                        break 'main_loop;
                    }

                    let dv = u32::from_le_bytes(src[src_idx..src_idx + 4].try_into().unwrap());
                    let dindex = (((0x1824429du32.wrapping_mul(dv)) >> (32 - 14)) & ((1 << 14) - 1))
                        as usize;
                    m_pos = src_start + dict[dindex] as usize;
                    dict[dindex] = (src_idx - src_start) as u16;

                    if dv == u32::from_le_bytes(src[m_pos..m_pos + 4].try_into().unwrap()) {
                        break;
                    }

                    src_idx += 1 + ((src_idx - ii) >> 5);
                }

                ii -= ti;
                ti = 0;
                let t = src_idx - ii;

                match t {
                    0..=3 => {
                        dst[dst_idx - 2] |= t as u8;
                    }
                    4..=18 => {
                        dst[dst_idx] = t as u8 - 3;
                        dst_idx += 1;
                    }
                    19.. => {
                        let mut tt = t - 18;
                        dst[dst_idx] = 0;
                        dst_idx += 1;

                        while tt > 255 {
                            tt -= 255;
                            dst[dst_idx] = 0;
                            dst_idx += 1;
                        }

                        dst[dst_idx] = tt as u8;
                        dst_idx += 1;
                    }
                }

                dst[dst_idx..dst_idx + t].copy_from_slice(&src[ii..ii + t]);
                dst_idx += t;

                let mut m_len = 4;

                while src[src_idx + m_len] == src[m_pos + m_len] {
                    m_len += 1;

                    if src[src_idx + m_len] != src[m_pos + m_len] {
                        break;
                    }

                    m_len += 1;

                    if src[src_idx + m_len] != src[m_pos + m_len] {
                        break;
                    }

                    m_len += 1;

                    if src[src_idx + m_len] != src[m_pos + m_len] {
                        break;
                    }

                    m_len += 1;

                    if src[src_idx + m_len] != src[m_pos + m_len] {
                        break;
                    }

                    m_len += 1;

                    if src[src_idx + m_len] != src[m_pos + m_len] {
                        break;
                    }

                    m_len += 1;

                    if src[src_idx + m_len] != src[m_pos + m_len] {
                        break;
                    }

                    m_len += 1;

                    if src[src_idx + m_len] != src[m_pos + m_len] {
                        break;
                    }

                    m_len += 1;

                    if src_idx + m_len >= src_start + src_len - 20 {
                        break;
                    }
                }

                let mut m_off = src_idx - m_pos;
                src_idx += m_len;
                ii = src_idx;

                if m_len <= 8 && m_off <= 0x0800 {
                    m_off -= 1;
                    dst[dst_idx] = (((m_len - 1) << 5) | ((m_off & 7) << 2)) as u8;
                    dst_idx += 1;
                    dst[dst_idx] = (m_off >> 3) as u8;
                    dst_idx += 1;
                } else if m_off <= 0x4000 {
                    m_off -= 1;

                    if m_len <= 33 {
                        dst[dst_idx] = (32 | (m_len - 2)) as u8;
                        dst_idx += 1;
                    } else {
                        m_len -= 33;
                        dst[dst_idx] = 32;
                        dst_idx += 1;

                        while m_len > 255 {
                            m_len -= 255;
                            dst[dst_idx] = 0;
                            dst_idx += 1;
                        }

                        dst[dst_idx] = m_len as u8;
                        dst_idx += 1;
                    }

                    dst[dst_idx] = (m_off << 2) as u8;
                    dst_idx += 1;
                    dst[dst_idx] = (m_off >> 6) as u8;
                    dst_idx += 1;
                } else {
                    m_off -= 0x4000;

                    if m_len <= 9 {
                        dst[dst_idx] = (16 | ((m_off >> 11) & 8) | (m_len - 2)) as u8;
                        dst_idx += 1;
                    } else {
                        m_len -= 9;
                        dst[dst_idx] = (16 | ((m_off >> 11) & 8)) as u8;
                        dst_idx += 1;

                        while m_len > 255 {
                            m_len -= 255;
                            dst[dst_idx] = 0;
                            dst_idx += 1;
                        }

                        dst[dst_idx] = m_len as u8;
                        dst_idx += 1;
                    }

                    dst[dst_idx] = (m_off << 2) as u8;
                    dst_idx += 1;
                    dst[dst_idx] = (m_off >> 6) as u8;
                    dst_idx += 1;
                }
            }

            ((src_start + src_len) - (ii - ti), dst_idx - dst_start)
        };

        t = new_t;

        src_idx += ll;
        dst_idx += out_len;
        l -= ll;
    }

    t += l;

    if t > 0 {
        let ii = src_len - t;

        if dst_idx == 0 && t < 238 {
            dst[dst_idx] = 17 + t as u8;
            dst_idx += 1;
        } else if t <= 3 {
            dst[dst_idx - 2] |= t as u8;
        } else if t <= 18 {
            dst[dst_idx] = t as u8 - 3;
            dst_idx += 1;
        } else {
            let mut tt = t - 18;

            dst[dst_idx] = 0;
            dst_idx += 1;

            while tt > 255 {
                tt -= 255;
                dst[dst_idx] = 0;
                dst_idx += 1;
            }

            dst[dst_idx] = tt as u8;
            dst_idx += 1;
        }

        dst[dst_idx..dst_idx + t].copy_from_slice(&src[ii..ii + t]);
        dst_idx += t;
    }

    dst[dst_idx] = 17;
    dst_idx += 1;
    dst[dst_idx] = 0;
    dst_idx += 1;
    dst[dst_idx] = 0;
    dst_idx += 1;

    dst.resize(dst_idx, 0);
    dst
}
