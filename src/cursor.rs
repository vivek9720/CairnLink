use crate::error::{DecodeError, ErrorKind, Result};

#[derive(Clone, Copy, Debug)]
pub struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    pub fn u8(&mut self) -> Result<u8> {
        if self.remaining() < 1 {
            return Err(DecodeError::eof(self.pos));
        }
        let v = self.data[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn i8(&mut self) -> Result<i8> {
        Ok(self.u8()? as i8)
    }

    pub fn u16(&mut self) -> Result<u16> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub fn i16(&mut self) -> Result<i16> {
        Ok(self.u16()? as i16)
    }

    pub fn u32(&mut self) -> Result<u32> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        if self.remaining() < n {
            return Err(DecodeError::eof(self.pos));
        }
        let start = self.pos;
        self.pos += n;
        Ok(&self.data[start..start + n])
    }

    pub fn take_len8(&mut self) -> Result<&'a [u8]> {
        let len = self.u8()? as usize;
        self.take(len)
    }

    pub fn take_len16(&mut self) -> Result<&'a [u8]> {
        let len = self.u16()? as usize;
        self.take(len)
    }

    pub fn varint(&mut self) -> Result<u32> {
        let mut shift = 0;
        let mut out = 0u32;
        for _ in 0..5 {
            let b = self.u8()?;
            out |= ((b & 0x7f) as u32) << shift;
            if b & 0x80 == 0 {
                return Ok(out);
            }
            shift += 7;
        }
        Err(DecodeError::new(
            ErrorKind::InvalidFrame,
            self.pos,
            "varint too long",
        ))
    }

    pub fn split_remaining(self) -> &'a [u8] {
        &self.data[self.pos..]
    }
}
