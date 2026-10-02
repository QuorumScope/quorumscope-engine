use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("protocol version cannot be 0")]
    ZeroProtocolVersion,
    #[error("ledger sequence cannot be 0")]
    ZeroLedgerSequence,
    #[error("invalid ledger hash length: expected 32 bytes, got {0}")]
    InvalidHashLength(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolVersion(u32);

impl ProtocolVersion {
    pub fn new(version: u32) -> Result<Self, LedgerError> {
        if version == 0 {
            return Err(LedgerError::ZeroProtocolVersion);
        }
        Ok(Self(version))
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LedgerSequence(u32);

impl LedgerSequence {
    pub fn new(sequence: u32) -> Result<Self, LedgerError> {
        if sequence == 0 {
            return Err(LedgerError::ZeroLedgerSequence);
        }
        Ok(Self(sequence))
    }

    pub fn get(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for LedgerSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LedgerHash([u8; 32]);

impl LedgerHash {
    pub fn new(hash: [u8; 32]) -> Self {
        Self(hash)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, LedgerError> {
        if slice.len() != 32 {
            return Err(LedgerError::InvalidHashLength(slice.len()));
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(slice);
        Ok(Self(hash))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for LedgerHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LedgerCloseTime(u64);

impl LedgerCloseTime {
    pub fn as_timestamp(&self) -> i64 {
        self.0 as i64
    }
}

impl LedgerCloseTime {
    pub fn new(unix_timestamp: u64) -> Self {
        Self(unix_timestamp)
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for LedgerCloseTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LedgerReference {
    pub sequence: LedgerSequence,
    pub hash: Option<LedgerHash>,
    pub close_time: Option<LedgerCloseTime>,
}
