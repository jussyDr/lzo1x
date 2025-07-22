use std::io::{self, Read};

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
    let mut decompressor = Decompressor::new(src);

    decompressor
        .read_exact(dst)
        .map_err(|_| DecompressError::InvalidInput)?;

    Ok(())
}

pub struct Decompressor<R> {
    reader: R,
    peeked_byte: Option<u8>,
    dict: Vec<u8>,
    state: Option<State>,
    op: Option<Op>,
}

#[derive(Clone, Copy)]
enum State {
    A,
    B,
    C,
}

#[derive(Clone, Copy)]
enum Op {
    Lit(LitOp),
    Match(MatchOp),
    MatchLit(MatchOp, LitOp),
}

#[derive(Clone, Copy)]
struct LitOp {
    len: usize,
}

#[derive(Clone, Copy)]
struct MatchOp {
    dist: usize,
    len: usize,
}

impl<R> Decompressor<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            peeked_byte: None,
            dict: vec![],
            state: None,
            op: None,
        }
    }
}

impl<R: Read> Decompressor<R> {
    fn read_byte(&mut self) -> io::Result<u8> {
        match self.peeked_byte {
            None => {
                let mut buf = [0];
                self.reader.read_exact(&mut buf)?;

                Ok(buf[0])
            }
            Some(byte) => {
                self.peeked_byte = None;

                Ok(byte)
            }
        }
    }

    fn peek_byte(&mut self) -> io::Result<u8> {
        match self.peeked_byte {
            None => {
                let byte = self.read_byte()?;

                self.peeked_byte = Some(byte);

                Ok(byte)
            }
            Some(byte) => Ok(byte),
        }
    }

    fn count_zeros(&mut self) -> io::Result<usize> {
        let mut count = 0;

        while self.peek_byte()? == 0 {
            self.read_byte()?;

            count += 1;
        }

        Ok(count)
    }
}

impl<R: Read> Read for Decompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let mut n = 0;

        loop {
            // (partially) execute the current `op`.
            if let Some(op) = self.op {
                match op {
                    Op::Lit(LitOp { len: lit_len }) => {
                        // copy literal.
                        let count = (buf.len() - n).min(lit_len);

                        for _ in 0..count {
                            let byte = self.read_byte()?;

                            buf[n] = byte;
                            n += 1;

                            self.dict.push(byte);
                        }

                        if count == lit_len {
                            self.op = None;
                        } else {
                            self.op = Some(Op::Lit(LitOp {
                                len: lit_len - count,
                            }));
                        }

                        if lit_len >= count {
                            return Ok(n);
                        }
                    }
                    Op::Match(MatchOp {
                        dist: match_dist,
                        len: match_len,
                    }) => {
                        // copy match.
                        let count = (buf.len() - n).min(match_len);

                        for _ in 0..count {
                            let byte = self.dict[self.dict.len() - match_dist];

                            buf[n] = byte;
                            n += 1;

                            self.dict.push(byte);
                        }

                        if count == match_len {
                            self.op = None;
                        } else {
                            self.op = Some(Op::Match(MatchOp {
                                dist: match_dist,
                                len: match_len - count,
                            }));
                        }

                        if count == match_len {
                            return Ok(n);
                        }
                    }
                    Op::MatchLit(
                        MatchOp {
                            dist: match_dist,
                            len: match_len,
                        },
                        LitOp { len: lit_len },
                    ) => {
                        // copy match.
                        let count = (buf.len() - n).min(match_len);

                        for _ in 0..count {
                            let byte = self.dict[self.dict.len() - match_dist];

                            buf[n] = byte;
                            n += 1;

                            self.dict.push(byte);
                        }

                        if count == match_len {
                            self.op = Some(Op::Lit(LitOp { len: lit_len }));
                        } else {
                            self.op = Some(Op::MatchLit(
                                MatchOp {
                                    dist: match_dist,
                                    len: match_len - count,
                                },
                                LitOp { len: lit_len },
                            ));
                        }

                        // copy literal.
                        let count = (buf.len() - n).min(lit_len);

                        if count == 0 {
                            return Ok(n);
                        }

                        for _ in 0..count {
                            let byte = self.read_byte()?;

                            buf[n] = byte;
                            n += 1;

                            self.dict.push(byte);
                        }

                        if count >= lit_len {
                            self.op = None;
                        } else {
                            self.op = Some(Op::Lit(LitOp {
                                len: lit_len - count,
                            }));
                        }

                        if lit_len >= count {
                            return Ok(n);
                        }
                    }
                }
            }

            // get next `op` depending on the current state.
            match self.state {
                None => {
                    let insn = self.peek_byte()?;

                    if insn <= 17 {
                        self.state = Some(State::A);
                    } else {
                        self.read_byte()?;

                        let lit_len = (insn - 17) as usize;
                        self.op = Some(Op::Lit(LitOp { len: lit_len }));

                        if insn <= 20 {
                            self.state = Some(State::B);
                        } else {
                            self.state = Some(State::C);
                        }
                    }
                }
                Some(state) => {
                    let insn = self.read_byte()?;

                    if insn <= 15 && matches!(state, State::A) {
                        let lit_len = if insn == 0 {
                            let count = self.count_zeros()?;

                            (count * 255) + (self.read_byte()? as usize) + 18
                        } else {
                            (insn + 3) as usize
                        };

                        self.op = Some(Op::Lit(LitOp { len: lit_len }));

                        self.state = Some(State::C);
                    } else {
                        let (match_len, match_dist, insn) = match insn {
                            0..=15 => match state {
                                State::A => unreachable!(),
                                State::B | State::C => {
                                    let (match_len, match_dist_offset) = match state {
                                        State::A => unreachable!(),
                                        State::B => (2, 1),
                                        State::C => (3, 2049),
                                    };

                                    let match_dist = ((self.read_byte()? as usize) << 2)
                                        + ((insn >> 2) as usize)
                                        + match_dist_offset;

                                    (match_len, match_dist, insn)
                                }
                            },
                            16..=31 => {
                                let match_len = if (insn & 0b00000111) == 0 {
                                    let count = self.count_zeros()?;

                                    (count * 255) + (self.read_byte()? as usize) + 9
                                } else {
                                    ((insn & 0b00000111) as usize) + 2
                                };

                                let sub_insn = self.read_byte()?;

                                let match_dist = ((((insn & 0b00001000) >> 3) as usize) << 14)
                                    + ((sub_insn >> 2) as usize)
                                    + ((self.read_byte()? as usize) << 6)
                                    + 16384;

                                if match_dist == 16384 {
                                    todo!("end")
                                }

                                (match_len, match_dist, sub_insn)
                            }
                            32..=63 => {
                                let match_len = if (insn & 0b00011111) == 0 {
                                    let count = self.count_zeros()?;

                                    (count * 255) + (self.read_byte()? as usize) + 33
                                } else {
                                    ((insn & 0b00011111) as usize) + 2
                                };

                                let sub_insn = self.read_byte()?;

                                let match_dist = ((self.read_byte()? as usize) << 6)
                                    + ((sub_insn >> 2) as usize)
                                    + 1;

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

                                let match_dist = ((self.read_byte()? as usize) << 3)
                                    + (((insn & 0b00011100) >> 2) as usize)
                                    + 1;

                                (match_len, match_dist, insn)
                            }
                        };

                        let match_op = MatchOp {
                            dist: match_dist,
                            len: match_len,
                        };

                        let lit_len = (insn & 0b00000011) as usize;

                        if lit_len == 0 {
                            self.op = Some(Op::Match(match_op));
                            self.state = Some(State::A);
                        } else {
                            self.op = Some(Op::MatchLit(match_op, LitOp { len: lit_len }));
                            self.state = Some(State::B);
                        }
                    }
                }
            }
        }
    }
}
