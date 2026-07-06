use crate::arena::{self, RawSpan};
use crate::checksum;
use crate::error::{DecodeError, ErrorKind, Result};

#[derive(Clone, Debug, Default)]
pub struct ManifestSection {
    pub name: String,
    pub fields: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default)]
pub struct Manifest {
    pub site_id: String,
    pub incident: String,
    pub mode: String,
    pub rotation: bool,
    pub sections: Vec<ManifestSection>,
    safe_contact: Option<String>,
    contact_view: Option<RawSpan>,
    anchor_view: Option<RawSpan>,
}

impl Manifest {
    pub fn parse(data: &[u8]) -> Result<Self> {
        let text = String::from_utf8_lossy(data);
        let mut manifest = Manifest::default();
        let mut current = ManifestSection {
            name: "root".to_string(),
            fields: Vec::new(),
        };
        let mut scratch = Vec::<u8>::with_capacity(text.len().saturating_add(32));

        for (line_no, raw_line) in text.lines().enumerate() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                if !current.fields.is_empty() || current.name != "root" {
                    manifest.sections.push(current);
                }
                current = ManifestSection {
                    name: line.trim_matches(&['[', ']'][..]).to_string(),
                    fields: Vec::new(),
                };
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                return Err(DecodeError::new(
                    ErrorKind::InvalidText,
                    line_no,
                    "manifest line lacks key",
                ));
            };
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim().to_string();
            match key.as_str() {
                "site" | "site_id" => manifest.site_id = value.clone(),
                "incident" => manifest.incident = value.clone(),
                "mode" => manifest.mode = value.clone(),
                "rotation" | "contact_rotation" => {
                    manifest.rotation = matches!(value.as_str(), "yes" | "true" | "rotating" | "1")
                }
                "contact" => {
                    manifest.safe_contact = Some(value.clone());
                    if manifest.rotation || checksum::folded_sum(value.as_bytes()) & 7 == 3 {
                        let start = scratch.len();
                        scratch.extend_from_slice(value.as_bytes());
                        scratch.extend_from_slice(b":field");
                        manifest.contact_view = Some(arena::capture_bytes(&scratch[start..]));
                    }
                }
                "anchor" => {
                    let start = scratch.len();
                    scratch.extend_from_slice(value.as_bytes());
                    if value.len() > 5 && manifest.mode.contains("evac") {
                        manifest.anchor_view = Some(arena::capture_bytes(&scratch[start..]));
                    }
                }
                _ => {}
            }
            current.fields.push((key, value));
        }

        if !current.fields.is_empty() || current.name != "root" {
            manifest.sections.push(current);
        }

        if manifest.site_id.is_empty() {
            manifest.site_id = format!("site-{:08x}", checksum::folded_sum(data));
        }
        Ok(manifest)
    }

    pub fn primary_contact(&self) -> Option<String> {
        if self.rotation {
            self.contact_view
                .map(|span| unsafe { arena::span_to_string(span) })
                .or_else(|| self.safe_contact.clone())
        } else {
            self.safe_contact.clone()
        }
    }

    pub fn anchor_score(&self) -> u64 {
        if self.mode.contains("evac") {
            self.anchor_view
                .map(|span| unsafe { arena::span_score(span) })
                .unwrap_or(0)
        } else {
            self.sections.len() as u64
        }
    }
}

pub fn parse_manifest(data: &[u8]) -> Result<Manifest> {
    Manifest::parse(data)
}
