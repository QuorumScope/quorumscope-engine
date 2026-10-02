use std::fmt;
use crate::ledger::{LedgerSequence, LedgerCloseTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImpactEvidenceKind {
    Direct,
    ProtocolDerived,
    RecentlyObserved,
    DependencyObserved,
    Inferred,
}

impl fmt::Display for ImpactEvidenceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImpactEvidenceKind::Direct => write!(f, "direct"),
            ImpactEvidenceKind::ProtocolDerived => write!(f, "protocol_derived"),
            ImpactEvidenceKind::RecentlyObserved => write!(f, "recently_observed"),
            ImpactEvidenceKind::DependencyObserved => write!(f, "dependency_observed"),
            ImpactEvidenceKind::Inferred => write!(f, "inferred"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservationWindow {
    pub first_ledger: LedgerSequence,
    pub last_ledger: LedgerSequence,
    pub first_close_time: Option<LedgerCloseTime>,
    pub last_close_time: Option<LedgerCloseTime>,
    pub provider: String,
    pub is_complete_for_range: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactEvidence {
    pub kind: ImpactEvidenceKind,
    pub description: String,
    pub window: ObservationWindow,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImpactEstimate {
    pub summary: String,
    pub evidence: Vec<ImpactEvidence>,
    pub overall_window: ObservationWindow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_impact_evidence_kind_display() {
        assert_eq!(ImpactEvidenceKind::Direct.to_string(), "direct");
        assert_eq!(ImpactEvidenceKind::Inferred.to_string(), "inferred");
    }
}
use crate::incident::IncidentId;
use crate::ledger::LedgerSequence;

#[derive(Debug, Clone)]
pub struct ImpactSnapshot {
    pub incident_id: IncidentId,
    pub ledger_sequence: LedgerSequence,
    pub total_frozen_accounts: u32,
    pub total_frozen_trustlines: u32,
    pub total_bypassed_txs: u32,
}
