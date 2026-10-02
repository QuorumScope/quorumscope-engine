use std::time::SystemTime;
use crate::network::NetworkId;
use crate::ledger::{LedgerSequence, LedgerHash, LedgerCloseTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceKind {
    StellarRpcCurrentState,
    StellarRpcLedgerHistory,
    Fixture,
    LocalTestNetwork,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidence {
    pub kind: SourceKind,
    pub network_id: NetworkId,
    pub ledger_sequence: LedgerSequence,
    pub ledger_hash: Option<LedgerHash>,
    pub ledger_close_time: Option<LedgerCloseTime>,
    /// Raw XDR bytes or a cryptographic digest representing the source XDR
    pub raw_xdr_digest: Vec<u8>,
    pub observation_timestamp: SystemTime,
    pub parser_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DataFreshness {
    pub source_ledger: LedgerSequence,
    pub latest_network_ledger: LedgerSequence,
}

impl DataFreshness {
    pub fn new(source_ledger: LedgerSequence, latest_network_ledger: LedgerSequence) -> Self {
        Self {
            source_ledger,
            latest_network_ledger,
        }
    }

    pub fn lag_ledgers(&self) -> u32 {
        let source = self.source_ledger.get();
        let latest = self.latest_network_ledger.get();
        if latest > source {
            latest - source
        } else {
            0
        }
    }

    pub fn is_stale(&self, threshold: u32) -> bool {
        self.lag_ledgers() > threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_freshness() {
        let source = LedgerSequence::new(100).unwrap();
        let latest = LedgerSequence::new(103).unwrap();
        
        let freshness = DataFreshness::new(source, latest);
        
        assert_eq!(freshness.lag_ledgers(), 3);
        assert!(!freshness.is_stale(3));
        assert!(freshness.is_stale(2));
    }
}
