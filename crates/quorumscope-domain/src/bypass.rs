use std::fmt;
use std::time::SystemTime;
use crate::network::NetworkId;
use crate::ledger::LedgerSequence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BypassHash([u8; 32]);

impl BypassHash {
    pub fn new(hash: [u8; 32]) -> Self {
        Self(hash)
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        if slice.len() != 32 {
            return Err("Bypass hash must be exactly 32 bytes");
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(slice);
        Ok(Self(hash))
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for BypassHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BypassAction {
    Add,
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BypassChangeResult {
    Changed,
    IdempotentNoop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BypassChange {
    pub network_id: NetworkId,
    pub ledger_sequence: LedgerSequence,
    pub bypass_hash: BypassHash,
    pub action: BypassAction,
    pub result: BypassChangeResult,
    pub evidence_ref: Option<String>,
    pub created_at: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bypass_hash_display() {
        let hash = BypassHash::new([0xcd; 32]);
        let s = hash.to_string();
        assert_eq!(s.len(), 64);
        assert!(s.starts_with("cdcd"));
    }

    #[test]
    fn test_bypass_hash_from_slice() {
        let valid = [0u8; 32];
        assert!(BypassHash::from_slice(&valid).is_ok());
        
        let invalid = [0u8; 31];
        assert!(BypassHash::from_slice(&invalid).is_err());
    }
}
