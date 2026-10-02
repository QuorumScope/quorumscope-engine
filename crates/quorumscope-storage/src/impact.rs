use crate::error::StorageError;
use quorumscope_domain::impact::ImpactSnapshot;
use sqlx::PgPool;

pub struct ImpactRepository {
    pool: PgPool,
}

impl ImpactRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_impact_snapshot(
        &self,
        snapshot: &ImpactSnapshot,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO impact_snapshots (incident_id, ledger_sequence, total_frozen_accounts, total_frozen_trustlines, total_bypassed_txs)
            VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(snapshot.incident_id.as_uuid())
            .bind(snapshot.ledger_sequence.get() as i64)
            .bind(snapshot.total_frozen_accounts as i64)
            .bind(snapshot.total_frozen_trustlines as i64)
            .bind(snapshot.total_bypassed_txs as i64)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
