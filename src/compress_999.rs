use crate::{
    config::{
        M1_MARKER, M1_MAX_OFFSET, M2_MAX_LEN, M2_MAX_OFFSET, M2_MIN_LEN, M3_MARKER, M3_MAX_LEN,
        M3_MAX_OFFSET, M4_MARKER, M4_MAX_LEN, M4_MAX_OFFSET, MX_MAX_OFFSET,
    },
    swd::{Swd, SWD_F, SWD_MAX_CHAIN, SWD_THRESHOLD},
};

pub fn compress_999(src: &[u8]) -> Vec<u8> {
    let mut dst = vec![0; src.len() + (src.len() / 16) + 64 + 3];

    let dst_len = unsafe { compress_internal(src, &mut dst, 2, 32, 128, SWD_F, 2048, 1) };

    dst.resize(dst_len, 0);
    dst
}

pub struct Compress<'a> {
    look: usize,
    m_len: usize,
    m_off: usize,
    last_m_len: usize,
    last_m_off: usize,
    bp: usize,
    pub src_idx: usize,
    pub src: &'a [u8],
    textsize: usize,
    codesize: usize,
    lit_bytes: usize,
    match_bytes: usize,
    lazy: usize,
    r1_lit: usize,
    r1_m_len: usize,
    m1a_m: usize,
    m1b_m: usize,
    m2_m: usize,
    m3_m: usize,
    m4_m: usize,
    lit1_r: usize,
    lit2_r: usize,
    lit3_r: usize,
}

unsafe fn compress_internal(
    src: &[u8],
    dst: &mut [u8],
    try_lazy_parm: i32,
    mut good_length: usize,
    mut max_lazy: usize,
    mut nice_length: usize,
    mut max_chain: usize,
    flags: u32,
) -> usize {
    let mut try_lazy: usize = try_lazy_parm as usize;

    if try_lazy_parm < 0 {
        try_lazy = 1;
    }

    if good_length == 0 {
        good_length = 32;
    }

    if max_lazy == 0 {
        max_lazy = 32;
    }

    if nice_length == 0 {
        nice_length = 0;
    }

    if max_chain == 0 {
        max_chain = SWD_MAX_CHAIN;
    }

    let c = &mut Compress {
        look: 0,
        m_len: 0,
        m_off: 0,
        last_m_len: 0,
        last_m_off: 0,
        bp: 0,
        src_idx: 0,
        src,
        textsize: 0,
        codesize: 0,
        lit_bytes: 0,
        match_bytes: 0,
        lazy: 0,
        r1_lit: 0,
        r1_m_len: 0,
        m1a_m: 0,
        m1b_m: 0,
        m2_m: 0,
        m3_m: 0,
        m4_m: 0,
        lit1_r: 0,
        lit2_r: 0,
        lit3_r: 0,
    };

    let mut dst_idx = 0;
    let mut ii = 0;
    let mut lit = 0;

    let mut swd = Swd::new(&mut *(c as *mut Compress));
    swd.use_best_off = flags & 1 != 0;

    if max_chain > 0 {
        swd.max_chain = max_chain;
    }

    if nice_length > 0 {
        swd.nice_length = nice_length;
    }

    find_match(c, &mut swd, 0, 0);

    let mut m_len;
    let mut m_off;

    while c.look > 0 {
        c.codesize = dst_idx;

        m_len = c.m_len;
        m_off = c.m_off;

        if lit == 0 {
            ii = c.bp;
        }

        if m_len < 2
            || (m_len == 2 && (m_off > M1_MAX_OFFSET || lit == 0 || lit >= 4))
            || (m_len == 2 && dst_idx == 0)
            || (dst_idx == 0 && lit == 0)
            || (m_len == M2_MIN_LEN && m_off > MX_MAX_OFFSET && lit >= 4)
        {
            m_len = 0;
        }

        if m_len == 0 {
            lit += 1;
            swd.max_chain = max_chain;
            find_match(c, &mut swd, 1, 0);
            continue;
        }

        if swd.use_best_off {
            better_match(&mut swd, &mut m_len, &mut m_off);
        }

        let mut ahead = 0usize;
        let l1;
        let max_ahead: usize;

        if try_lazy == 0 || m_len >= max_lazy {
            l1 = 0;
            max_ahead = 0;
        } else {
            l1 = len_of_coded_match(m_len, m_off, lit);
            max_ahead = try_lazy.min(l1 - 1);
        }

        let mut l2;

        if ahead < max_ahead && c.look > m_len {
            while ahead < max_ahead && c.look > m_len {
                if m_len >= good_length {
                    swd.max_chain = max_chain >> 2;
                } else {
                    swd.max_chain = max_chain;
                }

                find_match(c, &mut swd, 1, 0);
                ahead += 1;

                if c.m_len < m_len {
                    continue;
                }

                if c.m_len == m_len && c.m_off >= m_off {
                    continue;
                }

                if swd.use_best_off {
                    better_match(&mut swd, &mut c.m_len, &mut c.m_off);
                }

                l2 = len_of_coded_match(c.m_len, c.m_off, lit + ahead);

                if l2 == 0 {
                    continue;
                }

                let l3 = if dst_idx == 0 {
                    0
                } else {
                    len_of_coded_match(ahead, m_off, lit)
                };

                let lazy_match_min_gain = min_gain(ahead, lit, lit + ahead, l1, l2, l3);

                if c.m_len >= m_len + lazy_match_min_gain {
                    c.lazy += 1;

                    if l3 != 0 {
                        dst_idx = code_run(c, dst, dst_idx, src, ii, lit, ahead);
                        lit = 0;
                        dst_idx = code_match(c, dst, dst_idx, ahead, m_off);
                    } else {
                        lit += ahead;
                    }

                    break;
                }
            }
        } else {
            dst_idx = code_run(c, dst, dst_idx, src, ii, lit, m_len);
            lit = 0;

            dst_idx = code_match(c, dst, dst_idx, m_len, m_off);
            swd.max_chain = max_chain;
            find_match(c, &mut swd, m_len, 1 + ahead);
        }
    }

    /* store final run */
    if lit > 0 {
        dst_idx = store_run(c, dst, dst_idx, src, ii, lit);
    }

    dst[dst_idx] = M4_MARKER as u8 | 1;
    dst_idx += 1;
    dst[dst_idx] = 0;
    dst_idx += 1;
    dst[dst_idx] = 0;
    dst_idx += 1;

    c.codesize = dst_idx;

    dst_idx
}

fn store_run(
    c: &mut Compress,
    dst: &mut [u8],
    mut dst_idx: usize,
    src: &[u8],
    mut ii: usize,
    mut t: usize,
) -> usize {
    c.lit_bytes += t;

    if dst_idx == 0 && t <= 238 {
        dst[dst_idx] = (17 + t) as u8;
        dst_idx += 1;
    } else if t <= 3 {
        dst[dst_idx - 2] |= t as u8;
        c.lit1_r += 1;
    } else if t <= 18 {
        dst[dst_idx] = (t - 3) as u8;
        dst_idx += 1;
        c.lit2_r += 1;
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
        c.lit3_r += 1;
    }

    while t > 0 {
        dst[dst_idx] = src[ii];
        dst_idx += 1;
        ii += 1;
        t -= 1;
    }

    dst_idx
}

fn code_match(
    c: &mut Compress,
    dst: &mut [u8],
    mut dst_idx: usize,
    mut m_len: usize,
    mut m_off: usize,
) -> usize {
    let x_len = m_len;
    let x_off = m_off;

    c.match_bytes += m_len;

    if m_len == 2 {
        m_off -= 1;

        dst[dst_idx] = (M1_MARKER | ((m_off & 3) << 2)) as u8;
        dst_idx += 1;
        dst[dst_idx] = (m_off >> 2) as u8;
        dst_idx += 1;

        c.m1a_m += 1;
    } else if m_len <= M2_MAX_LEN && m_off <= M2_MAX_OFFSET {
        m_off -= 1;
        dst[dst_idx] = (((m_len - 1) << 5) | ((m_off & 7) << 2)) as u8;
        dst_idx += 1;
        dst[dst_idx] = (m_off >> 3) as u8;
        dst_idx += 1;

        c.m2_m += 1;
    } else if m_len == M2_MIN_LEN && m_off <= MX_MAX_OFFSET && c.r1_lit >= 4 {
        m_off -= 1 + M2_MAX_OFFSET;
        dst[dst_idx] = (M1_MARKER | ((m_off & 3) << 2)) as u8;
        dst_idx += 1;
        dst[dst_idx] = (m_off >> 2) as u8;
        dst_idx += 1;
        c.m1b_m += 1;
    } else if m_off <= M3_MAX_OFFSET {
        m_off -= 1;

        if m_len <= M3_MAX_LEN {
            dst[dst_idx] = (M3_MARKER | (m_len - 2)) as u8;
            dst_idx += 1;
        } else {
            m_len -= M3_MAX_LEN;
            dst[dst_idx] = M3_MARKER as u8;
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
        c.m3_m += 1;
    } else {
        m_off -= 0x4000;
        let k = (m_off & 0x4000) >> 11;
        if m_len <= M4_MAX_LEN {
            dst[dst_idx] = (M4_MARKER | k | (m_len - 2)) as u8;
            dst_idx += 1;
        } else {
            m_len -= M4_MAX_LEN;
            dst[dst_idx] = (M4_MARKER | k) as u8;
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
        c.m4_m += 1;
    }

    c.last_m_len = x_len;
    c.last_m_off = x_off;

    dst_idx
}

fn code_run(
    c: &mut Compress,
    dst: &mut [u8],
    mut dst_idx: usize,
    src: &[u8],
    ii: usize,
    lit: usize,
    m_len: usize,
) -> usize {
    if lit > 0 {
        dst_idx = store_run(c, dst, dst_idx, src, ii, lit);
        c.r1_m_len = m_len;
        c.r1_lit = lit;
    } else {
        c.r1_m_len = 0;
        c.r1_lit = 0;
    }

    dst_idx
}

fn better_match(swd: &mut Swd, m_len: &mut usize, m_off: &mut usize) {
    if *m_len <= M2_MIN_LEN {
        return;
    }

    if *m_off <= M2_MAX_OFFSET {
        return;
    }

    if *m_off > M2_MAX_OFFSET
        && *m_len > M2_MIN_LEN
        && *m_len <= M2_MAX_LEN + 1
        && swd.best_off[*m_len - 1] != 0
        && swd.best_off[*m_len - 1] <= M2_MAX_OFFSET
    {
        *m_len -= 1;
        *m_off = swd.best_off[*m_len];
        return;
    }

    if *m_off > M3_MAX_OFFSET
        && *m_len > M4_MAX_LEN
        && *m_len <= M2_MAX_LEN + 2
        && swd.best_off[*m_len - 2] != 0
        && swd.best_off[*m_len - 2] <= M2_MAX_OFFSET
    {
        *m_len -= 2;
        *m_off = swd.best_off[*m_len];
        return;
    }

    if *m_off > M3_MAX_OFFSET
        && *m_len > M4_MAX_LEN
        && *m_len <= M3_MAX_LEN + 1
        && swd.best_off[*m_len - 1] != 0
        && swd.best_off[*m_len - 1] <= M3_MAX_OFFSET
    {
        *m_len -= 1;
        *m_off = swd.best_off[*m_len];
    }
}
fn len_of_coded_match(mut m_len: usize, m_off: usize, lit: usize) -> usize {
    let mut n = 4;

    if m_len < 2 {
        return 0;
    }

    if m_len == 2 {
        return if m_off <= M1_MAX_OFFSET && lit > 0 && lit < 4 {
            2
        } else {
            0
        };
    }

    if m_len <= M2_MAX_LEN && m_off <= M2_MAX_OFFSET {
        return 2;
    }

    if m_len == M2_MIN_LEN && m_off <= MX_MAX_OFFSET && lit >= 4 {
        return 2;
    }

    if m_off <= M3_MAX_OFFSET {
        if m_len <= M3_MAX_LEN {
            return 3;
        }

        m_len -= M3_MAX_LEN;

        while m_len > 255 {
            m_len -= 255;
            n += 1;
        }

        return n;
    }

    if m_off <= M4_MAX_OFFSET {
        if m_len <= M4_MAX_LEN {
            return 3;
        }

        m_len -= M4_MAX_LEN;

        while m_len > 255 {
            m_len -= 255;
            n += 1;
        }

        return n;
    }

    0
}

fn min_gain(ahead: usize, lit1: usize, lit2: usize, l1: usize, l2: usize, l3: usize) -> usize {
    let mut lazy_match_min_gain;

    lazy_match_min_gain = ahead;

    if lit1 <= 3 {
        lazy_match_min_gain += if lit2 <= 3 { 0 } else { 2 };
    } else if lit1 <= 18 {
        lazy_match_min_gain += if lit2 <= 18 { 0 } else { 1 };
    }

    lazy_match_min_gain += (l2 - l1) * 2;

    if l3 != 0 {
        lazy_match_min_gain -= (ahead - l3) * 2;
    }

    if (lazy_match_min_gain as isize) < 0 {
        lazy_match_min_gain = 0;
    }

    lazy_match_min_gain
}

fn find_match(c: &mut Compress, s: &mut Swd, this_len: usize, skip: usize) {
    if skip > 0 {
        s.accept(this_len - skip);
        c.textsize += this_len - skip + 1;
    } else {
        c.textsize += this_len - skip;
    }

    s.m_len = SWD_THRESHOLD;
    s.m_off = 0;

    if s.use_best_off {
        s.best_pos.fill(0);
    }

    s.find_best();
    c.m_len = s.m_len;
    c.m_off = s.m_off;

    s.get_byte();

    if s.b_char < 0 {
        c.look = 0;
        c.m_len = 0;
    } else {
        c.look += 1;
    }

    c.bp = c.src_idx - c.look;
}
