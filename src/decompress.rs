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
            if let Some(op) = self.op {
                match op {
                    Op::Lit(LitOp { len: lit_len }) => {
                        if n + lit_len > buf.len() {
                            let count = buf.len() - n;

                            for _ in 0..count {
                                let byte = self.read_byte()?;

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = Some(Op::Lit(LitOp {
                                len: lit_len - count,
                            }));

                            return Ok(n);
                        } else if n + lit_len == buf.len() {
                            for _ in 0..lit_len {
                                let byte = self.read_byte()?;

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = None;

                            return Ok(n);
                        } else {
                            for _ in 0..lit_len {
                                let byte = self.read_byte()?;

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = None;
                        }
                    }
                    Op::Match(MatchOp {
                        dist: match_dist,
                        len: match_len,
                    }) => {
                        if n + match_len > buf.len() {
                            let count = buf.len() - n;

                            for _ in 0..count {
                                let byte = self.dict[self.dict.len() - match_dist];

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = Some(Op::Match(MatchOp {
                                dist: match_dist,
                                len: match_len - count,
                            }));

                            return Ok(n);
                        } else if n + match_len == buf.len() {
                            for _ in 0..match_len {
                                let byte = self.dict[self.dict.len() - match_dist];

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = None;

                            return Ok(n);
                        } else {
                            for _ in 0..match_len {
                                let byte = self.dict[self.dict.len() - match_dist];

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = None;
                        }
                    }
                    Op::MatchLit(
                        MatchOp {
                            dist: match_dist,
                            len: match_len,
                        },
                        LitOp { len: lit_len },
                    ) => {
                        if n + match_len > buf.len() {
                            let count = buf.len() - n;

                            for _ in 0..count {
                                let byte = self.dict[self.dict.len() - match_dist];

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = Some(Op::MatchLit(
                                MatchOp {
                                    dist: match_dist,
                                    len: match_len - count,
                                },
                                LitOp { len: lit_len },
                            ));

                            return Ok(n);
                        } else if n + match_len == buf.len() {
                            for _ in 0..match_len {
                                let byte = self.dict[self.dict.len() - match_dist];

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = Some(Op::Lit(LitOp { len: lit_len }));

                            return Ok(n);
                        } else {
                            for _ in 0..match_len {
                                let byte = self.dict[self.dict.len() - match_dist];

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }
                        }

                        if n + lit_len > buf.len() {
                            let count = buf.len() - n;

                            for _ in 0..count {
                                let byte = self.read_byte()?;

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = Some(Op::Lit(LitOp {
                                len: lit_len - count,
                            }));

                            return Ok(n);
                        } else if n + lit_len == buf.len() {
                            for _ in 0..lit_len {
                                let byte = self.read_byte()?;

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = None;

                            return Ok(n);
                        } else {
                            for _ in 0..lit_len {
                                let byte = self.read_byte()?;

                                buf[n] = byte;
                                n += 1;

                                self.dict.push(byte);
                            }

                            self.op = None;
                        }
                    }
                }
            }

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

                    match insn {
                        0..=15 => match state {
                            State::A => {
                                let lit_len = if insn == 0 {
                                    let count = self.count_zeros()?;

                                    (count * 255) + (self.read_byte()? as usize) + 18
                                } else {
                                    (insn + 3) as usize
                                };

                                self.op = Some(Op::Lit(LitOp { len: lit_len }));

                                self.state = Some(State::C);
                            }
                            State::B | State::C => {
                                let match_op = match state {
                                    State::A => unreachable!(),
                                    State::B => {
                                        let match_dist = ((self.read_byte()? as usize) << 2)
                                            + ((insn >> 2) as usize)
                                            + 1;

                                        MatchOp {
                                            dist: match_dist,
                                            len: 2,
                                        }
                                    }
                                    State::C => {
                                        let match_dist = ((self.read_byte()? as usize) << 2)
                                            + ((insn >> 2) as usize)
                                            + 2049;

                                        MatchOp {
                                            dist: match_dist,
                                            len: 3,
                                        }
                                    }
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
                        },
                        16..=31 => {
                            if (insn & 0b00000111) == 0 {
                                let count = self.count_zeros()?;

                                let match_len = (count * 255) + (self.read_byte()? as usize) + 9;

                                let sub_insn = self.read_byte()?;

                                let match_dist = ((((insn & 0b00001000) >> 3) as usize) << 14)
                                    + ((sub_insn >> 2) as usize)
                                    + ((self.read_byte()? as usize) << 6)
                                    + 16384;

                                if match_dist == 16384 {
                                    todo!("end")
                                }

                                let match_op = MatchOp {
                                    dist: match_dist,
                                    len: match_len,
                                };

                                let lit_len = (sub_insn & 0b00000011) as usize;

                                if lit_len == 0 {
                                    self.op = Some(Op::Match(match_op));

                                    self.state = Some(State::A);
                                } else {
                                    self.op = Some(Op::MatchLit(match_op, LitOp { len: lit_len }));

                                    self.state = Some(State::B);
                                }
                            } else {
                                let match_len = ((insn & 0b00000111) as usize) + 2;

                                let sub_insn = self.read_byte()?;

                                let match_dist = ((((insn & 0b00001000) >> 3) as usize) << 14)
                                    + ((sub_insn >> 2) as usize)
                                    + ((self.read_byte()? as usize) << 6)
                                    + 16384;

                                if match_dist == 16384 {
                                    todo!("end")
                                }

                                let match_op = MatchOp {
                                    dist: match_dist,
                                    len: match_len,
                                };

                                let lit_len = (sub_insn & 0b00000011) as usize;

                                if lit_len == 0 {
                                    self.op = Some(Op::Match(match_op));

                                    self.state = Some(State::A);
                                } else {
                                    self.op = Some(Op::MatchLit(match_op, LitOp { len: lit_len }));

                                    self.state = Some(State::B);
                                }
                            }
                        }
                        32..=63 => {
                            if (insn & 0b00011111) == 0 {
                                let count = self.count_zeros()?;

                                let match_len = (count * 255) + (self.read_byte()? as usize) + 33;

                                let sub_insn = self.read_byte()?;

                                let match_dist = ((self.read_byte()? as usize) << 6)
                                    + ((sub_insn >> 2) as usize)
                                    + 1;

                                let match_op = MatchOp {
                                    dist: match_dist,
                                    len: match_len,
                                };

                                let lit_len = (sub_insn & 0b00000011) as usize;

                                if lit_len == 0 {
                                    self.op = Some(Op::Match(match_op));

                                    self.state = Some(State::A);
                                } else {
                                    self.op = Some(Op::MatchLit(match_op, LitOp { len: lit_len }));

                                    self.state = Some(State::B);
                                }
                            } else {
                                let match_len = ((insn & 0b00011111) as usize) + 2;

                                let sub_insn = self.read_byte()?;

                                let match_dist = ((self.read_byte()? as usize) << 6)
                                    + ((sub_insn >> 2) as usize)
                                    + 1;

                                let match_op = MatchOp {
                                    dist: match_dist,
                                    len: match_len,
                                };

                                let lit_len = (sub_insn & 0b00000011) as usize;

                                if lit_len == 0 {
                                    self.op = Some(Op::Match(match_op));

                                    self.state = Some(State::A);
                                } else {
                                    self.op = Some(Op::MatchLit(match_op, LitOp { len: lit_len }));

                                    self.state = Some(State::B);
                                }
                            }
                        }
                        64..=127 => {
                            let match_len = if insn & 0b00100000 != 0 { 4 } else { 3 };

                            let match_dist = ((self.read_byte()? as usize) << 3)
                                + (((insn & 0b00011100) >> 2) as usize)
                                + 1;

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
                        128..=255 => {
                            let match_len = (((insn & 0b01100000) >> 5) as usize) + 5;

                            let match_dist = ((self.read_byte()? as usize) << 3)
                                + (((insn & 0b00011100) >> 2) as usize)
                                + 1;

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
}
