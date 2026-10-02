use crate::error::IndexerError;
use quorumscope_domain::reconciliation::ReconciliationGap;

pub struct ReconciliationEngine;

impl Default for ReconciliationEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ReconciliationEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn match_state(
        &self,
        expected_xdr: &str,
        actual_xdr: &str,
    ) -> Result<Option<ReconciliationGap>, IndexerError> {
        if expected_xdr != actual_xdr {
            Ok(Some(ReconciliationGap {
                id: uuid::Uuid::new_v4(),
                incident_id: quorumscope_domain::incident::IncidentId(uuid::Uuid::new_v4()),
                ledger_sequence: quorumscope_domain::ledger::LedgerSequence::new(1).unwrap(),
                expected_xdr: expected_xdr.to_string(),
                actual_xdr: actual_xdr.to_string(),
            }))
        } else {
            Ok(None)
        }
    }
}
