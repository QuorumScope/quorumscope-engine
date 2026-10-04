use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ReportEvent {
    pub ledger: i64,
    pub kind: String,
    pub evidence_ref: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportKey {
    pub key_id: String,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReportSnapshot {
    pub ledger: i64,
    pub frozen_accounts: i64,
    pub frozen_trustlines: i64,
    pub bypassed_transactions: i64,
}

/// Stored facts about one freeze episode. Nothing here is computed beyond what the database holds.
#[derive(Debug, Clone, Serialize)]
pub struct ReportData {
    pub incident_id: String,
    pub network: String,
    pub basis: String,
    pub status: String,
    pub opened_ledger: i64,
    pub closed_ledger: Option<i64>,
    pub keys: Vec<ReportKey>,
    pub events: Vec<ReportEvent>,
    pub snapshots: Vec<ReportSnapshot>,
}
