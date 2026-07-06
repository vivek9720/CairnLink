use crate::arena::{self, RawSpan};
use crate::cursor::Cursor;
use crate::error::Result;

#[derive(Clone, Debug, Default)]
pub struct FragmentReassembler {
    lanes: Vec<Vec<u8>>,
    pending_views: Vec<RawSpan>,
    stitched: Vec<Vec<u8>>,
    pub completed: usize,
}

impl FragmentReassembler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_ops(&mut self, payload: &[u8]) -> Result<Vec<Vec<u8>>> {
        let mut c = Cursor::new(payload);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x30 => {
                    let lane = c.u8()? as usize;
                    let bytes = c.take_len8()?.to_vec();
                    if self.lanes.len() <= lane {
                        self.lanes.resize_with(lane + 1, Vec::new);
                    }
                    self.lanes[lane].extend_from_slice(&bytes);
                }
                0x31 => {
                    let lane = c.u8()? as usize;
                    if let Some(buf) = self.lanes.get(lane) {
                        self.pending_views.push(arena::capture_bytes(buf));
                    }
                }
                0x32 => {
                    let lane = c.u8()? as usize;
                    if let Some(buf) = self.lanes.get_mut(lane) {
                        buf.clear();
                        buf.shrink_to_fit();
                    }
                }
                0x33 => {
                    self.materialize_pending();
                }
                0x34 => {
                    let mut scratch = c.take_len8()?.to_vec();
                    scratch.extend_from_slice(b":radio");
                    self.pending_views.push(arena::capture_bytes(&scratch));
                }
                _ => {
                    if c.remaining() > 0 {
                        let _ = c.take((op as usize) % c.remaining().min(16).max(1))?;
                    }
                }
            }
        }
        Ok(self.stitched.clone())
    }

    pub fn materialize_pending(&mut self) {
        for span in self.pending_views.drain(..) {
            if span.len > 0 {
                let bytes = unsafe { arena::span_to_vec(span) };
                if !bytes.is_empty() {
                    self.stitched.push(bytes);
                    self.completed += 1;
                }
            }
        }
    }
}
