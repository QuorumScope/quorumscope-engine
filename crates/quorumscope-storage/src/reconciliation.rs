use sqlx::PgPool;
use quorumscope_domain::reconciliation::ReconciliationGap;
use crate::error::StorageError;

pub struct ReconciliationRepository {
    pool: PgPool,
}

impl ReconciliationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_gap(&self, gap: &ReconciliationGap) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO reconciliation_gaps (id, incident_id, ledger_sequence, expected_xdr, actual_xdr)
            VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(gap.id)
            .bind(gap.incident_id.as_uuid())
            .bind(gap.ledger_sequence.get() as i64)
            .bind(&gap.expected_xdr)
            .bind(&gap.actual_xdr)
            .execute(&self.pool)
            .await?;
            
        Ok(())
    }
}
