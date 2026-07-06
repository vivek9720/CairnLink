use crate::checksum;
use crate::cursor::Cursor;
use crate::error::{DecodeError, ErrorKind, Result};
use crate::model::{Frame, FrameKind};

pub fn parse_frame(data: &[u8]) -> Result<Frame> {
    let frames = parse_frames(data)?;
    frames
        .into_iter()
        .next()
        .ok_or_else(|| DecodeError::new(ErrorKind::InvalidFrame, 0, "no frame found"))
}

pub fn parse_frames(data: &[u8]) -> Result<Vec<Frame>> {
    let mut c = Cursor::new(data);
    let mut frames = Vec::new();
    while c.remaining() >= 8 {
        let start = c.position();
        let raw_kind = c.u8()?;
        let flags = c.u8()?;
        let channel = c.u16()?;
        let len = c.u16()? as usize;
        let expected = c.u16()?;
        if len > c.remaining() {
            if frames.is_empty() {
                return Err(DecodeError::new(
                    ErrorKind::UnexpectedEof,
                    start,
                    "frame body truncated",
                ));
            }
            break;
        }
        let payload = c.take(len)?.to_vec();
        if flags & 0x40 != 0 && checksum::crc16(&payload) != expected {
            return Err(DecodeError::new(
                ErrorKind::BadChecksum,
                start,
                "frame checksum mismatch",
            ));
        }
        frames.push(Frame {
            kind: FrameKind::from_byte(raw_kind),
            raw_kind,
            flags,
            channel,
            checksum: expected,
            payload,
        });
        if frames.len() > 512 {
            return Err(DecodeError::new(
                ErrorKind::LimitExceeded,
                start,
                "too many frames",
            ));
        }
    }

    if frames.is_empty() && checksum::looks_like_text(data) {
        frames.push(Frame {
            kind: FrameKind::Manifest,
            raw_kind: 2,
            flags: 0,
            channel: 0,
            checksum: checksum::crc16(data),
            payload: data.to_vec(),
        });
    }

    Ok(frames)
}

pub fn encode_frame_for_seed(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    out.push(kind);
    out.push(0x40);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
    out.extend_from_slice(&checksum::crc16(payload).to_le_bytes());
    out.extend_from_slice(payload);
    out
}
