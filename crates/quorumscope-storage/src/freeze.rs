use sqlx::PgPool;
use quorumscope_domain::freeze::{DecodedFrozenKey, FrozenKeyKind};
use uuid::Uuid;
use crate::error::StorageError;

pub struct FreezeStateRepository {
    pool: PgPool,
}

impl FreezeStateRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn store_freeze_state(&self, incident_id: Uuid, ledger: i64, key: &DecodedFrozenKey) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO freeze_state (incident_id, ledger_sequence, kind, decoded_json, canonical_xdr)
            VALUES ($1, $2, $3, $4, $5)
            "#)
            .bind(incident_id)
            .bind(ledger)
            .bind(match key.kind {
                FrozenKeyKind::Account => "account",
                FrozenKeyKind::Trustline => "trustline",
                FrozenKeyKind::ContractData => "contract_data",
                FrozenKeyKind::ContractCode => "contract_code",
            })
            .bind(&key.decoded_json)
            .bind(&key.canonical_xdr)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
