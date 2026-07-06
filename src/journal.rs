use crate::arena::{self, RawSpan};
use crate::cursor::Cursor;
use crate::error::Result;
use crate::model::JournalSummary;

#[derive(Clone, Debug)]
struct JournalEvent {
    code: u16,
    amount: i32,
    note: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct JournalReplayer {
    events: Vec<JournalEvent>,
    checkpoints: Vec<RawSpan>,
    score: i64,
}

impl JournalReplayer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replay(&mut self, payload: &[u8]) -> Result<JournalSummary> {
        let mut c = Cursor::new(payload);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x60 => {
                    let code = c.u16()?;
                    let amount = c.i16()? as i32;
                    let note = c.take_len8()?.to_vec();
                    self.events.push(JournalEvent { code, amount, note });
                }
                0x61 => {
                    let idx = c.u16()? as usize;
                    if let Some(event) = self.events.get(idx % self.events.len().max(1)) {
                        self.checkpoints.push(arena::capture_bytes(&event.note));
                    }
                }
                0x62 => {
                    let selector = c.u8()?;
                    self.events
                        .retain(|event| (event.code as u8 ^ selector) & 1 == 0);
                    if selector & 0x80 != 0 {
                        self.events.shrink_to_fit();
                    }
                }
                0x63 => {
                    self.close_checkpoints();
                }
                0x64 => {
                    let local = c.take_len8()?.to_vec();
                    self.checkpoints.push(arena::capture_bytes(&local));
                }
                _ => {
                    if c.remaining() > 0 {
                        let _ = c.take((op as usize) % c.remaining().min(10).max(1))?;
                    }
                }
            }
        }
        Ok(JournalSummary {
            events: self.events.len(),
            checkpoints: self.checkpoints.len(),
            score: self.score,
        })
    }

    fn close_checkpoints(&mut self) {
        for span in self.checkpoints.drain(..) {
            self.score ^= unsafe { arena::span_score(span) as i64 };
        }
        for event in &self.events {
            self.score = self.score.saturating_add(event.amount as i64);
        }
    }
}

pub fn replay_journal(payload: &[u8]) -> Result<JournalSummary> {
    let mut replayer = JournalReplayer::new();
    replayer.replay(payload)
}
