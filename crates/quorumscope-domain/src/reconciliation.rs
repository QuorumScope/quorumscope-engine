use uuid::Uuid;
use crate::ledger::LedgerSequence;
use crate::incident::IncidentId;

#[derive(Debug, Clone)]
pub struct ReconciliationGap {
    pub id: Uuid,
    pub incident_id: IncidentId,
    pub ledger_sequence: LedgerSequence,
    pub expected_xdr: String,
    pub actual_xdr: String,
}
