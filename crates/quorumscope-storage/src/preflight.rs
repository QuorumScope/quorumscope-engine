use crate::error::StorageError;
use quorumscope_domain::preflight::{PreflightObservation, PreflightObservationResult};
use sqlx::PgPool;

pub struct PreflightRepository {
    pool: PgPool,
}

impl PreflightRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_observation(&self, obs: &PreflightObservation) -> Result<(), StorageError> {
        let result = match obs.result {
            PreflightObservationResult::Allowed => "allowed",
            PreflightObservationResult::Blocked => "blocked",
            PreflightObservationResult::Bypassed => "bypassed",
        };

        sqlx::query(
            r#"
            INSERT INTO preflight_observations (incident_id, ledger_sequence, transaction_hash, result)
            VALUES ($1, $2, $3, $4)
            "#)
            .bind(obs.incident_id.as_uuid())
            .bind(obs.ledger_sequence.get() as i64)
            .bind(&obs.transaction_hash)
            .bind(result)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
