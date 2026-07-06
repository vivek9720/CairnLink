use crate::arena::{self, RawSlice, RawSpan};
use crate::cursor::Cursor;
use crate::error::Result;
use crate::model::StockItem;

#[derive(Clone, Debug, Default)]
pub struct InventoryLedger {
    pub items: Vec<StockItem>,
    name_snapshots: Vec<RawSpan>,
    quantity_view: Option<RawSlice<i32>>,
    quantity_index: usize,
    pub movements: i64,
}

impl InventoryLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_ops(&mut self, payload: &[u8]) -> Result<()> {
        let mut c = Cursor::new(payload);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x20 => {
                    let sku = c.u16()?;
                    let quantity = c.i16()? as i32;
                    let expires_day = c.u16()?;
                    let bin = c.u8()?;
                    let name = c.take_len8()?.to_vec();
                    if self.items.len() < 2048 {
                        self.items.push(StockItem {
                            sku,
                            quantity,
                            expires_day,
                            bin,
                            name,
                        });
                    }
                }
                0x21 => {
                    let sku = c.u16()?;
                    let delta = c.i16()? as i32;
                    for item in &mut self.items {
                        if item.sku == sku {
                            item.quantity = item.quantity.saturating_add(delta);
                            self.movements += delta as i64;
                        }
                    }
                }
                0x22 => {
                    let idx = c.u16()? as usize;
                    if let Some(item) = self.items.get(idx % self.items.len().max(1)) {
                        self.name_snapshots.push(arena::capture_bytes(&item.name));
                    }
                }
                0x23 => {
                    let selector = c.u8()?;
                    self.rotate(selector);
                }
                0x24 => {
                    let quantities: Vec<i32> =
                        self.items.iter().map(|item| item.quantity).collect();
                    self.quantity_view = Some(arena::capture_slice(&quantities));
                    self.quantity_index = c.u16()? as usize;
                }
                0x25 => {
                    self.movements += self.reconcile() as i64;
                }
                _ => {
                    if c.remaining() > 0 {
                        let _ = c.take((op as usize) % c.remaining().min(12).max(1))?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn rotate(&mut self, selector: u8) {
        self.items.retain(|item| {
            if selector & 1 == 0 {
                item.quantity > 0
            } else {
                item.expires_day > selector as u16
            }
        });
        if selector & 0x80 != 0 {
            self.items.shrink_to_fit();
        }
    }

    pub fn reconcile(&self) -> i32 {
        let mut score = self.items.iter().map(|item| item.quantity).sum::<i32>();
        for span in &self.name_snapshots {
            if span.len > 8 {
                score ^= unsafe { arena::span_score(*span) as i32 };
            }
        }
        if let Some(raw) = self.quantity_view {
            if !raw.is_empty() && self.quantity_index >= raw.len {
                let value = unsafe { arena::read_at(raw, self.quantity_index) };
                score = score.saturating_add(value);
            }
        }
        score
    }
}

pub fn replay_inventory(payload: &[u8]) -> Result<InventoryLedger> {
    let mut ledger = InventoryLedger::new();
    ledger.parse_ops(payload)?;
    Ok(ledger)
}
