use crate::arena::{self, RawSlice};
use crate::cursor::Cursor;
use crate::error::Result;
use crate::model::{ScriptResult, TelemetryPoint};

#[derive(Clone, Debug, Default)]
pub struct ScriptVm {
    stack: Vec<i64>,
    telemetry: Vec<i16>,
    window: Option<RawSlice<i16>>,
    extra_window: usize,
    alerts: u16,
}

impl ScriptVm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn seed_from_points(&mut self, points: &[TelemetryPoint]) {
        for point in points {
            self.telemetry.push(point.occupancy as i16);
            self.telemetry.push(point.medical_wait as i16);
        }
    }

    pub fn run(&mut self, bytecode: &[u8]) -> Result<ScriptResult> {
        let mut c = Cursor::new(bytecode);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x50 => {
                    self.stack.push(c.i16()? as i64);
                }
                0x51 => {
                    let a = self.stack.pop().unwrap_or_default();
                    let b = self.stack.pop().unwrap_or_default();
                    self.stack.push(a.saturating_add(b));
                }
                0x52 => {
                    let value = c.i16()?;
                    if self.telemetry.len() < 4096 {
                        self.telemetry.push(value);
                    }
                }
                0x53 => {
                    self.window = Some(arena::capture_slice(&self.telemetry));
                    self.extra_window = c.u8()? as usize;
                }
                0x54 => {
                    let drop = (c.u8()? as usize).min(self.telemetry.len());
                    self.telemetry.drain(0..drop);
                    if c.peek().unwrap_or(0) & 1 == 1 {
                        self.telemetry.shrink_to_fit();
                    }
                }
                0x55 => {
                    if let Some(raw) = self.window {
                        let sum = unsafe { arena::slice_prefix_sum_i16(raw, self.extra_window) };
                        self.stack.push(sum);
                    }
                }
                0x56 => {
                    self.alerts = self.alerts.wrapping_add(c.u8()? as u16);
                }
                _ => {
                    if c.remaining() > 0 {
                        let _ = c.take((op as usize) % c.remaining().min(6).max(1))?;
                    }
                }
            }
        }
        Ok(ScriptResult {
            accumulator: self.stack.iter().copied().sum(),
            alerts: self.alerts,
            last_window: self.window.map(|w| w.len).unwrap_or_default(),
        })
    }
}

pub fn run_script(bytecode: &[u8]) -> Result<ScriptResult> {
    let mut vm = ScriptVm::new();
    vm.run(bytecode)
}
