use std::fmt;
use std::time::SystemTime;
use crate::network::NetworkId;
use crate::ledger::LedgerSequence;
use crate::evidence::SourceEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrozenKeyId([u8; 32]);

impl FrozenKeyId {
    pub fn new(hash: [u8; 32]) -> Self {
        Self(hash)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for FrozenKeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrozenKeyKind {
    Account,
    Trustline,
    ContractData,
    ContractCode,
}

impl fmt::Display for FrozenKeyKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FrozenKeyKind::Account => write!(f, "account"),
            FrozenKeyKind::Trustline => write!(f, "trustline"),
            FrozenKeyKind::ContractData => write!(f, "contract_data"),
            FrozenKeyKind::ContractCode => write!(f, "contract_code"),
        }
    }
}

/// Abstract representation of a decoded key.
/// Actual Stellar details will be defined in XDR or specialized domain types later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedFrozenKey {
    pub kind: FrozenKeyKind,
    pub decoded_json: String,
    pub canonical_xdr: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreezeAction {
    Freeze,
    Unfreeze,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreezeChangeResult {
    Changed,
    IdempotentNoop,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreezeChange {
    pub network_id: NetworkId,
    pub ledger_sequence: LedgerSequence,
    pub key_hash: FrozenKeyId,
    pub action: FreezeAction,
    pub result: FreezeChangeResult,
    pub evidence_ref: Option<String>,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreezeState {
    pub network_id: NetworkId,
    pub key_hash: FrozenKeyId,
    pub active_since: LedgerSequence,
    pub last_changed: LedgerSequence,
    pub evidence_ref: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frozen_key_kind_display() {
        assert_eq!(FrozenKeyKind::Account.to_string(), "account");
        assert_eq!(FrozenKeyKind::Trustline.to_string(), "trustline");
        assert_eq!(FrozenKeyKind::ContractData.to_string(), "contract_data");
        assert_eq!(FrozenKeyKind::ContractCode.to_string(), "contract_code");
    }

    #[test]
    fn test_frozen_key_id_display() {
        let hash = FrozenKeyId::new([0xab; 32]);
        let s = hash.to_string();
        assert_eq!(s.len(), 64);
        assert!(s.starts_with("abab"));
    }
}
