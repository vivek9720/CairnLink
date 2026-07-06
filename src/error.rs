use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    UnexpectedEof,
    BadMagic,
    BadChecksum,
    InvalidFrame,
    InvalidSection,
    InvalidText,
    InvalidScript,
    LimitExceeded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodeError {
    pub kind: ErrorKind,
    pub offset: usize,
    pub detail: &'static str,
}

pub type Result<T> = std::result::Result<T, DecodeError>;

impl DecodeError {
    pub fn new(kind: ErrorKind, offset: usize, detail: &'static str) -> Self {
        Self {
            kind,
            offset,
            detail,
        }
    }

    pub fn eof(offset: usize) -> Self {
        Self::new(ErrorKind::UnexpectedEof, offset, "unexpected end of input")
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {}: {}", self.kind, self.offset, self.detail)
    }
}

impl std::error::Error for DecodeError {}
