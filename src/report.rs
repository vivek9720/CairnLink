use crate::arena::{self, RawSpan};
use crate::cursor::Cursor;
use crate::dictionary::Dictionary;
use crate::error::Result;

#[derive(Clone, Debug, Default)]
pub struct TemplateBook {
    tokens: Vec<String>,
    retained: Vec<RawSpan>,
    rendered: Vec<String>,
}

impl TemplateBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_ops(&mut self, payload: &[u8], dict: &Dictionary) -> Result<()> {
        let mut c = Cursor::new(payload);
        while !c.is_empty() {
            let op = c.u8()?;
            match op {
                0x70 => {
                    self.tokens
                        .push(String::from_utf8_lossy(c.take_len8()?).into_owned());
                }
                0x71 => {
                    let id = c.u16()?;
                    if let Some(value) = dict.resolve_alias_text(id) {
                        self.retained.push(arena::capture_bytes(value.as_bytes()));
                    }
                }
                0x72 => {
                    let mut local = c.take_len8()?.to_vec();
                    local.extend_from_slice(b":template");
                    self.retained.push(arena::capture_bytes(&local));
                }
                0x73 => {
                    self.render();
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

    pub fn render(&mut self) {
        let mut line = self.tokens.join(" ");
        for span in self.retained.drain(..) {
            let text = unsafe { arena::span_to_string(span) };
            line.push(' ');
            line.push_str(&text);
        }
        if !line.is_empty() {
            self.rendered.push(line);
        }
    }

    pub fn rendered_count(&self) -> usize {
        self.rendered.len()
    }
}
