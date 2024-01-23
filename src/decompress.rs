use crate::DecompressError;

#[derive(Debug)]
enum State {
    A,
    B,
    C,
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
    let mut src_pos = 0;
    let mut dst_pos = 0;

    let instruction = src[src_pos];

    let mut state = match instruction {
        0..=17 => State::A,
        18..=20 => {
            src_pos += 1;

            let length = (instruction as usize) - 17;

            for _ in 0..length {
                dst[dst_pos] = src[src_pos];
                src_pos += 1;
                dst_pos += 1;
            }

            State::B
        }
        21..=255 => {
            src_pos += 1;

            let length = (instruction as usize) - 17;

            for _ in 0..length {
                dst[dst_pos] = src[src_pos];
                src_pos += 1;
                dst_pos += 1;
            }

            State::C
        }
    };

    loop {
        let instruction = src[src_pos];
        src_pos += 1;

        state = match instruction {
            0..=15 => match state {
                State::A => {
                    if instruction == 0 {
                        let start = src_pos;

                        while src[src_pos] == 0 {
                            src_pos += 1;
                        }

                        let count = src_pos - start;

                        let length = (count * 255) + (src[src_pos] as usize) + 18;
                        src_pos += 1;

                        for _ in 0..length {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::C
                    } else {
                        let length = (instruction as usize) + 3;

                        for _ in 0..length {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::C
                    }
                }
                State::B => {
                    let distance =
                        ((src[src_pos] as usize) << 2) + ((instruction >> 2) as usize) + 1;
                    src_pos += 1;

                    let mut match_pos = dst_pos - distance;

                    for _ in 0..2 {
                        dst[dst_pos] = dst[match_pos];
                        match_pos += 1;
                        dst_pos += 1;
                    }

                    let state = instruction & 0b00000011;

                    if state == 0 {
                        State::A
                    } else {
                        for _ in 0..state {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::B
                    }
                }
                State::C => {
                    let distance =
                        ((src[src_pos] as usize) << 2) + ((instruction >> 2) as usize) + 2049;
                    src_pos += 1;

                    let mut match_pos = dst_pos - distance;

                    for _ in 0..3 {
                        dst[dst_pos] = dst[match_pos];
                        match_pos += 1;
                        dst_pos += 1;
                    }

                    let state = instruction & 0b00000011;

                    if state == 0 {
                        State::A
                    } else {
                        for _ in 0..state {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::B
                    }
                }
            },
            16..=31 => {
                if (instruction & 0b00000111) == 0 {
                    let start = src_pos;

                    while src[src_pos] == 0 {
                        src_pos += 1;
                    }

                    let count = src_pos - start;

                    let length = (count * 255) + (src[src_pos] as usize) + 9;
                    src_pos += 1;

                    let distance = ((((instruction & 0b00001000) >> 3) as usize) << 14)
                        + ((src[src_pos + 1] as usize) << 6)
                        + ((src[src_pos] >> 2) as usize)
                        + 16384;

                    if distance == 16384 {
                        break;
                    }

                    let state = src[src_pos] & 0b00000011;
                    src_pos += 2;

                    let mut match_pos = dst_pos - distance;

                    for _ in 0..length {
                        dst[dst_pos] = dst[match_pos];
                        match_pos += 1;
                        dst_pos += 1;
                    }

                    if state == 0 {
                        State::A
                    } else {
                        for _ in 0..state {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::B
                    }
                } else {
                    let length = ((instruction & 0b00000111) as usize) + 2;

                    let distance = ((((instruction & 0b00001000) >> 3) as usize) << 14)
                        + ((src[src_pos + 1] as usize) << 6)
                        + ((src[src_pos] >> 2) as usize)
                        + 16384;

                    if distance == 16384 {
                        break;
                    }

                    let state = src[src_pos] & 0b00000011;
                    src_pos += 2;

                    let mut match_pos = dst_pos - distance;

                    for _ in 0..length {
                        dst[dst_pos] = dst[match_pos];
                        match_pos += 1;
                        dst_pos += 1;
                    }

                    if state == 0 {
                        State::A
                    } else {
                        for _ in 0..state {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::B
                    }
                }
            }
            32..=63 => {
                if (instruction & 0b00011111) == 0 {
                    let start = src_pos;

                    while src[src_pos] == 0 {
                        src_pos += 1;
                    }

                    let count = src_pos - start;

                    let length = (count * 255) + (src[src_pos] as usize) + 33;
                    src_pos += 1;

                    let distance =
                        ((src[src_pos + 1] as usize) << 6) + ((src[src_pos] >> 2) as usize) + 1;
                    let state = src[src_pos] & 0b00000011;
                    src_pos += 2;

                    let mut match_pos = dst_pos - distance;

                    for _ in 0..length {
                        dst[dst_pos] = dst[match_pos];
                        match_pos += 1;
                        dst_pos += 1;
                    }

                    if state == 0 {
                        State::A
                    } else {
                        for _ in 0..state {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::B
                    }
                } else {
                    let length = ((instruction & 0b00011111) as usize) + 2;

                    let distance =
                        ((src[src_pos + 1] as usize) << 6) + ((src[src_pos] >> 2) as usize) + 1;
                    let state = src[src_pos] & 0b00000011;
                    src_pos += 2;

                    let mut match_pos = dst_pos - distance;

                    for _ in 0..length {
                        dst[dst_pos] = dst[match_pos];
                        match_pos += 1;
                        dst_pos += 1;
                    }

                    if state == 0 {
                        State::A
                    } else {
                        for _ in 0..state {
                            dst[dst_pos] = src[src_pos];
                            src_pos += 1;
                            dst_pos += 1;
                        }

                        State::B
                    }
                }
            }
            64..=127 => {
                let length = (((instruction & 0b00100000) >> 5) as usize) + 3;

                let distance = ((src[src_pos] as usize) << 3)
                    + (((instruction & 0b00011100) >> 2) as usize)
                    + 1;
                src_pos += 1;

                let mut match_pos = dst_pos - distance;

                for _ in 0..length {
                    dst[dst_pos] = dst[match_pos];
                    match_pos += 1;
                    dst_pos += 1;
                }

                let state = instruction & 0b00000011;

                if state == 0 {
                    State::A
                } else {
                    for _ in 0..state {
                        dst[dst_pos] = src[src_pos];
                        src_pos += 1;
                        dst_pos += 1;
                    }

                    State::B
                }
            }
            128..=255 => {
                let length = (((instruction & 0b01100000) >> 5) as usize) + 5;

                let distance = ((src[src_pos] as usize) << 3)
                    + (((instruction & 0b00011100) >> 2) as usize)
                    + 1;
                src_pos += 1;

                let mut match_pos = dst_pos - distance;

                for _ in 0..length {
                    dst[dst_pos] = dst[match_pos];
                    match_pos += 1;
                    dst_pos += 1;
                }

                let state = instruction & 0b00000011;

                if state == 0 {
                    State::A
                } else {
                    for _ in 0..state {
                        dst[dst_pos] = src[src_pos];
                        src_pos += 1;
                        dst_pos += 1;
                    }

                    State::B
                }
            }
        }
    }

    Ok(())
}
