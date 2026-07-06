use crate::arena::{self, RawSpan};
use crate::catalog;
use crate::cursor::Cursor;
use crate::error::Result;

#[derive(Clone, Debug)]
pub struct AliasRecord {
    pub id: u16,
    pub target: usize,
    pub span: RawSpan,
    pub generation: u32,
    pub weight: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Dictionary {
    entries: Vec<Vec<u8>>,
    aliases: Vec<AliasRecord>,
    generation: u32,
    resolved_log: Vec<String>,
}

impl Dictionary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn add_entry(&mut self, value: &[u8]) {
        if self.entries.len() < 4096 {
            self.entries.push(value.to_vec());
        }
    }

    pub fn add_catalog_entry(&mut self, seed: usize) {
        let phrase = catalog::phrase(seed as u32);
        self.add_entry(phrase.as_bytes());
    }

    pub fn add_alias(&mut self, id: u16, target: usize, weight: u8) {
        if self.entries.is_empty() {
            return;
        }
        let idx = target % self.entries.len();
        let span = arena::capture_bytes(&self.entries[idx]);
        self.aliases.push(AliasRecord {
            id,
            target: idx,
            span,
            generation: self.generation,
            weight,
        });
    }

    pub fn compact_for_locale(&mut self, selector: u8) {
        self.generation = self.generation.wrapping_add(1);
        let keep_even = selector & 1 == 0;
        let mut fresh = Vec::with_capacity(self.entries.len().saturating_add(1));
        for (idx, entry) in self.entries.iter().enumerate() {
            if (idx % 2 == 0) == keep_even || entry.len() < 4 {
                let mut next = entry.clone();
                if selector & 0x20 != 0 {
                    next.make_ascii_uppercase();
                }
                fresh.push(next);
            }
        }
        if fresh.is_empty() {
            fresh.push(format!("fallback-{}", selector).into_bytes());
        }
        self.entries = fresh;
    }

    pub fn resolve_alias_text(&self, id: u16) -> Option<String> {
        let alias = self.aliases.iter().rev().find(|a| a.id == id)?;
        if alias.generation == self.generation {
            self.entries
                .get(alias.target % self.entries.len().max(1))
                .map(|v| String::from_utf8_lossy(v).into_owned())
        } else if alias.weight & 1 == 1 {
            Some(unsafe { arena::span_to_string(alias.span) })
        } else {
            self.entries
                .get(alias.target % self.entries.len().max(1))
                .map(|v| String::from_utf8_lossy(v).into_owned())
        }
    }

    pub fn audit_aliases(&mut self) -> u64 {
        let mut score = 0u64;
        for alias in &self.aliases {
            if alias.generation != self.generation && alias.weight & 0x4 != 0 {
                score ^= unsafe { arena::span_score(alias.span) };
            } else {
                score = score.wrapping_add(alias.id as u64 + alias.weight as u64);
            }
        }
        score
    }

    pub fn sample(&self, idx: usize) -> Option<&[u8]> {
        self.entries
            .get(idx % self.entries.len().max(1))
            .map(Vec::as_slice)
    }

    pub fn parse_ops(&mut self, payload: &[u8]) -> Result<()> {
        let mut c = Cursor::new(payload);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x01 => {
                    let text = c.take_len8()?;
                    self.add_entry(text);
                }
                0x02 => {
                    let id = c.u16()?;
                    let target = c.u8()? as usize;
                    let weight = c.u8()?;
                    self.add_alias(id, target, weight);
                }
                0x03 => {
                    let selector = c.u8()?;
                    self.compact_for_locale(selector);
                }
                0x04 => {
                    let id = c.u16()?;
                    if let Some(text) = self.resolve_alias_text(id) {
                        self.resolved_log.push(text);
                    }
                }
                0x05 => {
                    self.entries.clear();
                    self.entries.shrink_to_fit();
                    self.generation = self.generation.wrapping_add(1);
                }
                0x06 => {
                    let seed = c.u16()? as usize;
                    self.add_catalog_entry(seed);
                }
                0x07 => {
                    let id = c.u16()?;
                    let seed = c.u16()? as usize;
                    self.add_catalog_entry(seed);
                    self.add_alias(
                        id,
                        self.entries.len().saturating_sub(1),
                        (seed & 0xff) as u8,
                    );
                }
                _ => {
                    if c.remaining() > 0 {
                        let skip = (op as usize) % c.remaining().min(16).max(1);
                        let _ = c.take(skip)?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn decode_dictionary(payload: &[u8]) -> Result<Dictionary> {
    let mut dict = Dictionary::new();
    dict.parse_ops(payload)?;
    Ok(dict)
}
