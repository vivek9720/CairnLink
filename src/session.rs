use crate::analyzer;
use crate::bundle;
use crate::codec::BitBackrefDecoder;
use crate::dictionary::Dictionary;
use crate::error::Result;
use crate::fragment::FragmentReassembler;
use crate::frame::parse_frames;
use crate::inventory::InventoryLedger;
use crate::journal;
use crate::manifest::Manifest;
use crate::model::{Frame, FrameKind, JournalSummary, ScriptResult, TelemetryPoint};
use crate::report::TemplateBook;
use crate::route::RouteBook;
use crate::script::ScriptVm;

#[derive(Clone, Debug)]
pub struct SessionState {
    pub dictionary: Dictionary,
    pub manifest: Manifest,
    pub routes: RouteBook,
    pub inventory: InventoryLedger,
    pub fragments: FragmentReassembler,
    pub reports: TemplateBook,
    pub telemetry: Vec<TelemetryPoint>,
    pub journal_summaries: Vec<JournalSummary>,
    pub script_results: Vec<ScriptResult>,
    pub codec_output: Vec<u8>,
    pub risk_score: i64,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            dictionary: Dictionary::new(),
            manifest: Manifest::default(),
            routes: RouteBook::new(),
            inventory: InventoryLedger::new(),
            fragments: FragmentReassembler::new(),
            reports: TemplateBook::new(),
            telemetry: Vec::new(),
            journal_summaries: Vec::new(),
            script_results: Vec::new(),
            codec_output: Vec::new(),
            risk_score: 0,
        }
    }

    pub fn apply_frame(&mut self, frame: Frame) -> Result<()> {
        match frame.kind {
            FrameKind::Dictionary => self.dictionary.parse_ops(&frame.payload)?,
            FrameKind::Manifest => self.manifest = Manifest::parse(&frame.payload)?,
            FrameKind::Route => self.routes.parse_ops(&frame.payload, &self.dictionary)?,
            FrameKind::Inventory => self.inventory.parse_ops(&frame.payload)?,
            FrameKind::Fragment => {
                let stitched = self.fragments.parse_ops(&frame.payload)?;
                for bytes in stitched {
                    if let Ok(nested) = parse_frames(&bytes) {
                        for nested_frame in nested {
                            self.apply_frame(nested_frame)?;
                        }
                    }
                }
            }
            FrameKind::Script => {
                let mut vm = ScriptVm::new();
                vm.seed_from_points(&self.telemetry);
                self.script_results.push(vm.run(&frame.payload)?);
            }
            FrameKind::Journal => self
                .journal_summaries
                .push(journal::replay_journal(&frame.payload)?),
            FrameKind::Report => self.reports.parse_ops(&frame.payload, &self.dictionary)?,
            FrameKind::Codec => {
                let mut decoder = BitBackrefDecoder::new();
                self.codec_output
                    .extend(decoder.decode_tile(&frame.payload)?);
            }
            FrameKind::Telemetry => self.parse_telemetry(&frame.payload)?,
            FrameKind::Bundle => {
                if frame.payload.starts_with(b"CIRN") {
                    let nested = bundle::decode_bundle(&frame.payload)?;
                    self.risk_score ^= nested.risk_score;
                }
            }
            FrameKind::Unknown(_) => {
                if frame.flags & 1 != 0 {
                    self.dictionary.parse_ops(&frame.payload)?;
                }
            }
        }
        Ok(())
    }

    pub fn parse_telemetry(&mut self, payload: &[u8]) -> Result<()> {
        let mut c = crate::cursor::Cursor::new(payload);
        while c.remaining() >= 13 {
            self.telemetry.push(TelemetryPoint {
                shelter_id: c.u16()?,
                minute: c.u32()?,
                occupancy: c.u16()?,
                water_liters: c.u16()?,
                medical_wait: c.u16()?,
                flags: c.u8()?,
            });
            if self.telemetry.len() > 2048 {
                break;
            }
        }
        Ok(())
    }

    pub fn finalize(&mut self) -> Result<()> {
        let report = analyzer::analyze_session(self);
        self.risk_score = report.score;
        Ok(())
    }
}

pub fn decode_stream(data: &[u8]) -> Result<SessionState> {
    if data.starts_with(b"CIRN") {
        return bundle::decode_bundle(data);
    }
    let mut state = SessionState::new();
    let frames = parse_frames(data)?;
    for frame in frames {
        state.apply_frame(frame)?;
    }
    state.finalize()?;
    Ok(state)
}
