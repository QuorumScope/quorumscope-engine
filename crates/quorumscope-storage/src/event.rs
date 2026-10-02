use crate::error::StorageError;
use quorumscope_domain::incident::IncidentEvent;
use sqlx::PgPool;

pub struct IncidentEventRepository {
    pool: PgPool,
}

impl IncidentEventRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_event(&self, event: &IncidentEvent) -> Result<(), StorageError> {
        let kind = match event.kind {
            quorumscope_domain::incident::IncidentEventKind::FreezeChange => "freeze_change",
            quorumscope_domain::incident::IncidentEventKind::BypassChange => "bypass_change",
            quorumscope_domain::incident::IncidentEventKind::Reconciliation => "reconciliation",
            quorumscope_domain::incident::IncidentEventKind::ImpactSnapshot => "impact_snapshot",
            quorumscope_domain::incident::IncidentEventKind::PreflightObservation => {
                "preflight_observation"
            }
        };

        sqlx::query(
            r#"
            INSERT INTO incident_events (incident_id, ledger_sequence, kind, evidence_ref)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(event.incident_id.as_uuid())
        .bind(event.ledger_sequence.get() as i64)
        .bind(kind)
        .bind(&event.evidence_ref)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
