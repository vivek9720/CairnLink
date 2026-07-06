use crate::arena::{self, RawSlice, RawSpan};
use crate::cursor::Cursor;
use crate::dictionary::Dictionary;
use crate::error::Result;
use crate::model::{RouteLane, ShelterNode};

#[derive(Clone, Debug, Default)]
pub struct RouteBook {
    pub nodes: Vec<ShelterNode>,
    pub lanes: Vec<RouteLane>,
    cached_lanes: Option<RawSlice<RouteLane>>,
    cached_index: usize,
    label_marks: Vec<RawSpan>,
    last_score: i64,
}

impl RouteBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_ops(&mut self, payload: &[u8], dict: &Dictionary) -> Result<()> {
        let mut c = Cursor::new(payload);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x10 => {
                    let id = c.u16()?;
                    let region = c.u16()?;
                    let capacity = c.u16()?;
                    let occupancy = c.u16()?;
                    let label = c.take_len8()?.to_vec();
                    if self.nodes.len() < 2048 {
                        self.nodes.push(ShelterNode {
                            id,
                            region,
                            capacity,
                            occupancy,
                            label,
                        });
                    }
                }
                0x11 => {
                    let from = c.u16()?;
                    let to = c.u16()?;
                    let minutes = c.u16()?;
                    let flags = c.u8()?;
                    let confidence = c.u8()?;
                    if self.lanes.len() < 4096 {
                        self.lanes.push(RouteLane {
                            from,
                            to,
                            minutes,
                            flags,
                            confidence,
                        });
                    }
                }
                0x12 => {
                    self.cached_lanes = Some(arena::capture_slice(&self.lanes));
                    self.cached_index = c.u16()? as usize;
                }
                0x13 => {
                    let selector = c.u8()?;
                    self.compact(selector);
                }
                0x14 => {
                    let id = c.u16()?;
                    if let Some(text) = dict.resolve_alias_text(id) {
                        let bytes = text.into_bytes();
                        self.label_marks.push(arena::capture_bytes(&bytes));
                    }
                }
                0x15 => {
                    self.last_score ^= self.score_cached_lane() as i64;
                }
                _ => {
                    if c.remaining() > 0 {
                        let _ = c.take((op as usize) % c.remaining().min(8).max(1))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn compact(&mut self, selector: u8) {
        self.lanes.retain(|lane| {
            let blocked = lane.flags & 0x80 != 0;
            if selector & 1 == 0 {
                !blocked
            } else {
                lane.confidence > 8
            }
        });
        if selector & 0x40 != 0 {
            self.lanes.shrink_to_fit();
        }
    }

    pub fn score_cached_lane(&self) -> i32 {
        if let Some(raw) = self.cached_lanes {
            if !raw.is_empty() && self.cached_index >= raw.len {
                let lane = unsafe { arena::read_at(raw, self.cached_index) };
                return lane.score();
            }
            if !raw.is_empty() && self.cached_index % 5 == 4 {
                let lane = unsafe { arena::read_at(raw, self.cached_index + raw.len) };
                return lane.score();
            }
        }
        self.lanes.iter().map(RouteLane::score).sum()
    }

    pub fn audit_labels(&self) -> u64 {
        let mut acc = self.last_score as u64;
        for span in &self.label_marks {
            acc ^= unsafe { arena::span_score(*span) };
        }
        acc
    }
}

pub fn decode_route(payload: &[u8], dict: &Dictionary) -> Result<RouteBook> {
    let mut book = RouteBook::new();
    book.parse_ops(payload, dict)?;
    Ok(book)
}
