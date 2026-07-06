use crate::arena::{self, RawSpan};
use crate::cursor::Cursor;
use crate::error::Result;

#[derive(Clone, Debug, Default)]
pub struct BitBackrefDecoder {
    history: Vec<u8>,
    marks: Vec<RawSpan>,
    pub produced: Vec<u8>,
}

impl BitBackrefDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decode_tile(&mut self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut c = Cursor::new(payload);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x40 => {
                    let bytes = c.take_len8()?;
                    self.history.extend_from_slice(bytes);
                    self.produced.extend_from_slice(bytes);
                }
                0x41 => {
                    let start = c.u8()? as usize;
                    let len = c.u8()? as usize;
                    if start < self.history.len() {
                        let end = (start + len).min(self.history.len());
                        self.marks
                            .push(arena::capture_bytes(&self.history[start..end]));
                    }
                }
                0x42 => {
                    self.history.clear();
                    self.history.shrink_to_fit();
                }
                0x43 => {
                    let idx = c.u8()? as usize;
                    let repeat = c.u8()? as usize;
                    if let Some(span) = self.marks.get(idx % self.marks.len().max(1)).copied() {
                        let src = unsafe { arena::span_to_vec(span) };
                        for _ in 0..repeat.min(8) {
                            self.produced.extend_from_slice(&src);
                        }
                    }
                }
                0x44 => {
                    let mut local = c.take_len8()?.to_vec();
                    local.reverse();
                    self.marks.push(arena::capture_bytes(&local));
                }
                _ => {
                    if c.remaining() > 0 {
                        let _ = c.take((op as usize) % c.remaining().min(8).max(1))?;
                    }
                }
            }
        }
        Ok(self.produced.clone())
    }
}
