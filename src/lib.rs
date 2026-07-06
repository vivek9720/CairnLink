//! CairnLink decodes offline disaster-shelter exchange bundles.
//!
//! The project deliberately keeps dependencies out of the build so fuzzing can run
//! hermetically. Public entry points parse multi-stage structured inputs and then
//! drive decoded state through reconciliation and analysis.

pub mod analyzer;
pub mod arena;
pub mod bundle;
pub mod catalog;
pub mod checksum;
pub mod codec;
pub mod cursor;
pub mod dictionary;
pub mod error;
pub mod fragment;
pub mod frame;
pub mod inventory;
pub mod journal;
pub mod manifest;
pub mod model;
pub mod report;
pub mod route;
pub mod script;
pub mod session;

pub use analyzer::{analyze_session, RiskReport};
pub use bundle::{decode_bundle, BundleDecoder};
pub use error::{DecodeError, ErrorKind, Result};
pub use frame::{parse_frame, parse_frames};
pub use journal::replay_journal;
pub use script::run_script;
pub use session::{decode_stream, SessionState};

pub fn parse(data: &[u8]) -> Result<SessionState> {
    decode_stream(data)
}

pub fn parse_one(data: &[u8]) -> Result<model::Frame> {
    parse_frame(data)
}
