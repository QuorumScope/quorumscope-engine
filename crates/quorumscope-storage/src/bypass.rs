use crate::error::StorageError;
use quorumscope_domain::bypass::DecodedBypassTransaction;
use sqlx::PgPool;
use uuid::Uuid;

pub struct BypassStateRepository {
    pool: PgPool,
}

impl BypassStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_bypass_state(
        &self,
        incident_id: Uuid,
        ledger: i64,
        tx: &DecodedBypassTransaction,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO bypass_state (incident_id, ledger_sequence, tx_hash, decoded_json, canonical_xdr)
            VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(incident_id)
            .bind(ledger)
            .bind(&tx.hash)
            .bind(&tx.decoded_json)
            .bind(&tx.canonical_xdr)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
