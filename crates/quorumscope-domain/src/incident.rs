use std::fmt;
use uuid::Uuid;
use crate::network::NetworkId;
use crate::ledger::{LedgerSequence, LedgerCloseTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IncidentId(Uuid);

impl IncidentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for IncidentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for IncidentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncidentStatus {
    Active,
    Resolved,
}

impl fmt::Display for IncidentStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncidentStatus::Active => write!(f, "active"),
            IncidentStatus::Resolved => write!(f, "resolved"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Incident {
    pub id: IncidentId,
    pub network_id: NetworkId,
    pub basis: String, // Fixed to `freeze_set_episode` as per requirements
    pub opened_ledger: LedgerSequence,
    pub opened_close_time: Option<LedgerCloseTime>,
    pub closed_ledger: Option<LedgerSequence>,
    pub closed_close_time: Option<LedgerCloseTime>,
    pub status: IncidentStatus,
}

impl Incident {
    pub fn new(
        network_id: NetworkId,
        opened_ledger: LedgerSequence,
        opened_close_time: Option<LedgerCloseTime>,
    ) -> Self {
        Self {
            id: IncidentId::new(),
            network_id,
            basis: "freeze_set_episode".to_string(),
            opened_ledger,
            opened_close_time,
            closed_ledger: None,
            closed_close_time: None,
            status: IncidentStatus::Active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncidentEventKind {
    FreezeChange,
    BypassChange,
    Reconciliation,
    ImpactSnapshot,
    PreflightObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncidentEvent {
    pub incident_id: IncidentId,
    pub ledger_sequence: LedgerSequence,
    pub kind: IncidentEventKind,
    pub evidence_ref: String, // e.g., the hash or ID of the specific event
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incident_status_display() {
        assert_eq!(IncidentStatus::Active.to_string(), "active");
        assert_eq!(IncidentStatus::Resolved.to_string(), "resolved");
    }

    #[test]
    fn test_incident_basis() {
        let network_id = NetworkId::new();
        let ledger = LedgerSequence::new(100).unwrap();
        let incident = Incident::new(network_id, ledger, None);
        assert_eq!(incident.basis, "freeze_set_episode");
        assert_eq!(incident.status, IncidentStatus::Active);
        assert!(incident.closed_ledger.is_none());
    }
}
