#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    Dictionary,
    Manifest,
    Route,
    Inventory,
    Fragment,
    Script,
    Journal,
    Report,
    Codec,
    Telemetry,
    Bundle,
    Unknown(u8),
}

impl FrameKind {
    pub fn from_byte(value: u8) -> Self {
        match value % 16 {
            0 | 1 => FrameKind::Dictionary,
            2 => FrameKind::Manifest,
            3 => FrameKind::Route,
            4 => FrameKind::Inventory,
            5 => FrameKind::Fragment,
            6 => FrameKind::Script,
            7 => FrameKind::Journal,
            8 => FrameKind::Report,
            9 => FrameKind::Codec,
            10 => FrameKind::Telemetry,
            11 => FrameKind::Bundle,
            other => FrameKind::Unknown(other),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub kind: FrameKind,
    pub raw_kind: u8,
    pub flags: u8,
    pub channel: u16,
    pub checksum: u16,
    pub payload: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TelemetryPoint {
    pub shelter_id: u16,
    pub minute: u32,
    pub occupancy: u16,
    pub water_liters: u16,
    pub medical_wait: u16,
    pub flags: u8,
}

#[derive(Clone, Debug)]
pub struct ShelterNode {
    pub id: u16,
    pub region: u16,
    pub capacity: u16,
    pub occupancy: u16,
    pub label: Vec<u8>,
}

impl ShelterNode {
    pub fn pressure(&self) -> i32 {
        self.occupancy as i32 - self.capacity as i32
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RouteLane {
    pub from: u16,
    pub to: u16,
    pub minutes: u16,
    pub flags: u8,
    pub confidence: u8,
}

impl RouteLane {
    pub fn score(&self) -> i32 {
        let blocked = if self.flags & 0x80 != 0 { 700 } else { 0 };
        self.minutes as i32 + blocked - self.confidence as i32
    }
}

#[derive(Clone, Debug)]
pub struct StockItem {
    pub sku: u16,
    pub quantity: i32,
    pub expires_day: u16,
    pub bin: u8,
    pub name: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct JournalSummary {
    pub events: usize,
    pub checkpoints: usize,
    pub score: i64,
}

#[derive(Clone, Debug, Default)]
pub struct ScriptResult {
    pub accumulator: i64,
    pub alerts: u16,
    pub last_window: usize,
}
