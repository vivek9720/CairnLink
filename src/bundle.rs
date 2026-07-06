use crate::cursor::Cursor;
use crate::error::{DecodeError, ErrorKind, Result};
use crate::model::{Frame, FrameKind};
use crate::session::SessionState;

#[derive(Clone, Debug, Default)]
pub struct BundleDecoder {
    pub sections: usize,
}

impl BundleDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn decode(&mut self, data: &[u8]) -> Result<SessionState> {
        let mut state = SessionState::new();
        let mut c = Cursor::new(data);
        if c.remaining() >= 4 && c.take(4)? != b"CIRN" {
            return Err(DecodeError::new(
                ErrorKind::BadMagic,
                0,
                "bundle magic mismatch",
            ));
        }
        let _version = c.u8().unwrap_or(1);
        let declared = c.u8().unwrap_or(0) as usize;
        let _flags = c.u16().unwrap_or(0);
        let limit = declared.max(1).min(128);
        for _ in 0..limit {
            if c.remaining() < 4 {
                break;
            }
            let kind = c.u8()?;
            let flags = c.u8()?;
            let len = c.u16()? as usize;
            if len > c.remaining() {
                break;
            }
            let payload = c.take(len)?.to_vec();
            let frame = Frame {
                kind: FrameKind::from_byte(kind),
                raw_kind: kind,
                flags,
                channel: self.sections as u16,
                checksum: 0,
                payload,
            };
            state.apply_frame(frame)?;
            self.sections += 1;
        }
        state.finalize()?;
        Ok(state)
    }
}

pub fn decode_bundle(data: &[u8]) -> Result<SessionState> {
    let mut decoder = BundleDecoder::new();
    decoder.decode(data)
}
