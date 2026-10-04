use quorumscope_domain::bypass::{BypassChange, BypassHash};
use quorumscope_domain::freeze::{FreezeChange, FrozenKeyId};
use quorumscope_domain::network::NetworkId;
use sqlx::PgPool;
use std::collections::HashSet;

#[derive(Clone)]
pub struct IndexerRepository {
    pool: PgPool,
}

impl IndexerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub async fn get_checkpoint(
        &self,
        network_id: NetworkId,
        stream: &str,
    ) -> sqlx::Result<Option<i64>> {
        let row: Option<(i64,)> = sqlx::query_as(
            "SELECT last_complete_ledger FROM ingestion_checkpoints WHERE network_id = $1 AND stream = $2"
        )
        .bind(network_id.as_uuid())
        .bind(stream)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn upsert_checkpoint(
        &self,
        network_id: NetworkId,
        stream: &str,
        ledger: i64,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO ingestion_checkpoints (network_id, stream, last_complete_ledger, updated_at) 
             VALUES ($1, $2, $3, NOW()) 
             ON CONFLICT (network_id, stream) DO UPDATE 
             SET last_complete_ledger = EXCLUDED.last_complete_ledger, updated_at = NOW()"
        )
        .bind(network_id.as_uuid())
        .bind(stream)
        .bind(ledger)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_current_frozen_keys(
        &self,
        network_id: NetworkId,
    ) -> sqlx::Result<HashSet<FrozenKeyId>> {
        let rows: Vec<(Vec<u8>,)> =
            sqlx::query_as("SELECT key_hash FROM current_frozen_keys WHERE network_id = $1")
                .bind(network_id.as_uuid())
                .fetch_all(&self.pool)
                .await?;

        let mut set = HashSet::new();
        for (bytes,) in rows {
            if let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice()) {
                set.insert(FrozenKeyId::new(arr));
            }
        }
        Ok(set)
    }

    pub async fn get_current_bypasses(
        &self,
        network_id: NetworkId,
    ) -> sqlx::Result<HashSet<BypassHash>> {
        let rows: Vec<(String,)> =
            sqlx::query_as("SELECT tx_hash FROM current_bypasses WHERE network_id = $1")
                .bind(network_id.as_uuid())
                .fetch_all(&self.pool)
                .await?;

        let mut set = HashSet::new();
        for (hex_str,) in rows {
            let mut byte_hash = [0u8; 32];
            if let Ok(bytes) = hex::decode(&hex_str)
                && let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice())
            {
                byte_hash = arr;
            }
            set.insert(BypassHash::new(byte_hash));
        }
        Ok(set)
    }

    pub async fn apply_freeze_changes(&self, changes: &[FreezeChange]) -> sqlx::Result<()> {
        let mut tx = self.pool.begin().await?;

        for change in changes {
            let action_str = match change.action {
                quorumscope_domain::freeze::FreezeAction::Freeze => "freeze",
                quorumscope_domain::freeze::FreezeAction::Unfreeze => "unfreeze",
            };

            let result_str = match change.result {
                quorumscope_domain::freeze::FreezeChangeResult::Changed => "changed",
                quorumscope_domain::freeze::FreezeChangeResult::IdempotentNoop => "idempotent_noop",
            };

            let id = uuid::Uuid::new_v4();

            sqlx::query(
                "INSERT INTO freeze_changes (id, network_id, ledger_sequence, key_hash, action, result, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, NOW())"
            )
            .bind(id)
            .bind(change.network_id.as_uuid())
            .bind(change.ledger_sequence.get() as i64)
            .bind(change.key_hash.as_bytes().as_slice())
            .bind(action_str)
            .bind(result_str)
            .execute(&mut *tx)
            .await?;

            if change.action == quorumscope_domain::freeze::FreezeAction::Freeze {
                sqlx::query(
                    "INSERT INTO current_frozen_keys (network_id, key_hash, active_since, last_changed)
                     VALUES ($1, $2, $3, $3)
                     ON CONFLICT (network_id, key_hash) DO UPDATE SET last_changed = EXCLUDED.last_changed"
                )
                .bind(change.network_id.as_uuid())
                .bind(change.key_hash.as_bytes().as_slice())
                .bind(change.ledger_sequence.get() as i64)
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query(
                    "DELETE FROM current_frozen_keys WHERE network_id = $1 AND key_hash = $2",
                )
                .bind(change.network_id.as_uuid())
                .bind(change.key_hash.as_bytes().as_slice())
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn apply_bypass_changes(&self, changes: &[BypassChange]) -> sqlx::Result<()> {
        let mut tx = self.pool.begin().await?;

        for change in changes {
            let action_str = match change.action {
                quorumscope_domain::bypass::BypassAction::Add => "add",
                quorumscope_domain::bypass::BypassAction::Remove => "remove",
            };

            let result_str = match change.result {
                quorumscope_domain::bypass::BypassChangeResult::Changed => "changed",
                quorumscope_domain::bypass::BypassChangeResult::IdempotentNoop => "idempotent_noop",
            };

            let id = uuid::Uuid::new_v4();

            sqlx::query(
                "INSERT INTO bypass_changes (id, network_id, ledger_sequence, tx_hash, action, result, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, NOW())"
            )
            .bind(id)
            .bind(change.network_id.as_uuid())
            .bind(change.ledger_sequence.get() as i64)
            .bind(hex::encode(change.bypass_hash.as_bytes()))
            .bind(action_str)
            .bind(result_str)
            .execute(&mut *tx)
            .await?;

            if change.action == quorumscope_domain::bypass::BypassAction::Add {
                sqlx::query(
                    "INSERT INTO current_bypasses (network_id, tx_hash, active_since, last_changed)
                     VALUES ($1, $2, $3, $3)
                     ON CONFLICT (network_id, tx_hash) DO UPDATE SET last_changed = EXCLUDED.last_changed"
                )
                .bind(change.network_id.as_uuid())
                .bind(hex::encode(change.bypass_hash.as_bytes()))
                .bind(change.ledger_sequence.get() as i64)
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query("DELETE FROM current_bypasses WHERE network_id = $1 AND tx_hash = $2")
                    .bind(change.network_id.as_uuid())
                    .bind(hex::encode(change.bypass_hash.as_bytes()))
                    .execute(&mut *tx)
                    .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn store_config_snapshot(
        &self,
        network_id: NetworkId,
        ledger: i64,
        setting_id: &str,
        xdr: &[u8],
        parsed_json: &serde_json::Value,
    ) -> sqlx::Result<()> {
        let id = uuid::Uuid::new_v4();
        sqlx::query(
            "INSERT INTO config_snapshots (id, network_id, ledger_sequence, config_setting_id, raw_entry_xdr, parsed_json, source_kind, observed_timestamp)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())"
        )
        .bind(id)
        .bind(network_id.as_uuid())
        .bind(ledger)
        .bind(setting_id)
        .bind(xdr)
        .bind(parsed_json)
        .bind("rpc")
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
