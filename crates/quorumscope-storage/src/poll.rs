use quorumscope_domain::bypass::{BypassAction, BypassChange, BypassHash};
use quorumscope_domain::freeze::{
    DecodedFrozenKey, FreezeAction, FreezeChange, FreezeChangeResult, FrozenKeyId,
};
use quorumscope_domain::network::NetworkId;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

/// A configuration entry read from the network, stored as evidence.
pub struct SnapshotRecord {
    pub id: Uuid,
    pub setting_id: &'static str,
    pub raw_xdr: Vec<u8>,
    pub parsed_json: serde_json::Value,
}

impl SnapshotRecord {
    pub fn evidence_ref(&self) -> String {
        format!("config_snapshot:{}", self.id)
    }
}

/// Everything learned from one read of the network, applied atomically.
pub struct PollObservation {
    pub network_id: NetworkId,
    pub ledger: i64,
    pub latest_network_ledger: i64,
    pub protocol_version: Option<i32>,
    pub snapshots: Vec<SnapshotRecord>,
    pub observed_keys: Vec<(FrozenKeyId, DecodedFrozenKey)>,
    pub observed_bypasses: Vec<BypassHash>,
    pub freeze_changes: Vec<FreezeChange>,
    pub bypass_changes: Vec<BypassChange>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PollOutcome {
    pub incident_opened: bool,
    pub incident_closed: bool,
    pub reconciled: bool,
}

fn freeze_action(action: FreezeAction) -> &'static str {
    match action {
        FreezeAction::Freeze => "freeze",
        FreezeAction::Unfreeze => "unfreeze",
    }
}

fn freeze_result(result: FreezeChangeResult) -> &'static str {
    match result {
        FreezeChangeResult::Changed => "changed",
        FreezeChangeResult::IdempotentNoop => "idempotent_noop",
    }
}

#[derive(Clone)]
pub struct PollRepository {
    pool: PgPool,
}

impl PollRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Records the latest network ledger without changing freeze state.
    pub async fn record_network_observation(
        &self,
        network_id: NetworkId,
        latest_network_ledger: i64,
        protocol_version: Option<i32>,
    ) -> sqlx::Result<()> {
        let mut tx = self.pool.begin().await?;
        upsert_observation(&mut tx, network_id, latest_network_ledger, protocol_version).await?;
        tx.commit().await
    }

    pub async fn apply(&self, obs: &PollObservation) -> sqlx::Result<PollOutcome> {
        let network = obs.network_id.as_uuid();
        let mut tx = self.pool.begin().await?;

        let has_changes = !obs.freeze_changes.is_empty() || !obs.bypass_changes.is_empty();
        for snap in &obs.snapshots {
            // An unchanged entry is not stored again, so repeated polls do not grow history.
            // Changes always keep their evidence snapshot.
            if !has_changes {
                let unchanged: Option<bool> = sqlx::query_scalar(
                    "SELECT raw_entry_xdr = $3 FROM config_snapshots WHERE network_id=$1 AND config_setting_id=$2 \
                     ORDER BY ledger_sequence DESC, observed_timestamp DESC LIMIT 1",
                )
                .bind(network)
                .bind(snap.setting_id)
                .bind(&snap.raw_xdr)
                .fetch_optional(&mut *tx)
                .await?;
                if unchanged == Some(true) {
                    continue;
                }
            }
            sqlx::query(
                "INSERT INTO config_snapshots (id, network_id, ledger_sequence, config_setting_id, raw_entry_xdr, parsed_json, source_kind, observed_timestamp) \
                 VALUES ($1, $2, $3, $4, $5, $6, 'rpc', NOW())",
            )
            .bind(snap.id)
            .bind(network)
            .bind(obs.ledger)
            .bind(snap.setting_id)
            .bind(&snap.raw_xdr)
            .bind(&snap.parsed_json)
            .execute(&mut *tx)
            .await?;
        }

        for (id, key) in &obs.observed_keys {
            sqlx::query(
                "INSERT INTO ledger_keys (network_id, key_hash, canonical_key_xdr, key_kind, decoded_json, first_observed_ledger, last_observed_ledger) \
                 VALUES ($1, $2, $3, $4, $5::jsonb, $6, $6) \
                 ON CONFLICT (network_id, key_hash) DO UPDATE SET last_observed_ledger = GREATEST(ledger_keys.last_observed_ledger, EXCLUDED.last_observed_ledger)",
            )
            .bind(network)
            .bind(id.as_bytes().as_slice())
            .bind(&key.canonical_xdr)
            .bind(key.kind.to_string())
            .bind(&key.decoded_json)
            .bind(obs.ledger)
            .execute(&mut *tx)
            .await?;
        }

        for change in &obs.freeze_changes {
            sqlx::query(
                "INSERT INTO freeze_changes (id, network_id, ledger_sequence, key_hash, action, result, evidence_ref, created_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())",
            )
            .bind(Uuid::new_v4())
            .bind(network)
            .bind(change.ledger_sequence.get() as i64)
            .bind(change.key_hash.as_bytes().as_slice())
            .bind(freeze_action(change.action))
            .bind(freeze_result(change.result))
            .bind(&change.evidence_ref)
            .execute(&mut *tx)
            .await?;
            if change.action == FreezeAction::Freeze {
                sqlx::query(
                    "INSERT INTO current_frozen_keys (network_id, key_hash, active_since, last_changed, evidence_ref) \
                     VALUES ($1, $2, $3, $3, $4) \
                     ON CONFLICT (network_id, key_hash) DO UPDATE SET last_changed = EXCLUDED.last_changed, evidence_ref = EXCLUDED.evidence_ref",
                )
                .bind(network)
                .bind(change.key_hash.as_bytes().as_slice())
                .bind(change.ledger_sequence.get() as i64)
                .bind(&change.evidence_ref)
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query(
                    "DELETE FROM current_frozen_keys WHERE network_id = $1 AND key_hash = $2",
                )
                .bind(network)
                .bind(change.key_hash.as_bytes().as_slice())
                .execute(&mut *tx)
                .await?;
            }
        }

        for change in &obs.bypass_changes {
            let action = match change.action {
                BypassAction::Add => "add",
                BypassAction::Remove => "remove",
            };
            let hash = change.bypass_hash.to_string();
            sqlx::query(
                "INSERT INTO bypass_changes (id, network_id, ledger_sequence, tx_hash, action, result, evidence_ref, created_at) \
                 VALUES ($1, $2, $3, $4, $5, 'changed', $6, NOW())",
            )
            .bind(Uuid::new_v4())
            .bind(network)
            .bind(change.ledger_sequence.get() as i64)
            .bind(&hash)
            .bind(action)
            .bind(&change.evidence_ref)
            .execute(&mut *tx)
            .await?;
            if change.action == BypassAction::Add {
                sqlx::query(
                    "INSERT INTO current_bypasses (network_id, tx_hash, active_since, last_changed, evidence_ref) \
                     VALUES ($1, $2, $3, $3, $4) \
                     ON CONFLICT (network_id, tx_hash) DO UPDATE SET last_changed = EXCLUDED.last_changed, evidence_ref = EXCLUDED.evidence_ref",
                )
                .bind(network)
                .bind(&hash)
                .bind(change.ledger_sequence.get() as i64)
                .bind(&change.evidence_ref)
                .execute(&mut *tx)
                .await?;
            } else {
                sqlx::query("DELETE FROM current_bypasses WHERE network_id = $1 AND tx_hash = $2")
                    .bind(network)
                    .bind(&hash)
                    .execute(&mut *tx)
                    .await?;
            }
        }

        let (active_keys, active_bypasses): (i64, i64) = sqlx::query_as(
            "SELECT (SELECT COUNT(*) FROM current_frozen_keys WHERE network_id=$1), \
                    (SELECT COUNT(*) FROM current_bypasses WHERE network_id=$1)",
        )
        .bind(network)
        .fetch_one(&mut *tx)
        .await?;
        let mut incident: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM incidents WHERE network_id=$1 AND status='active' ORDER BY opened_ledger DESC LIMIT 1",
        )
        .bind(network)
        .fetch_optional(&mut *tx)
        .await?;
        let mut outcome = PollOutcome {
            incident_opened: false,
            incident_closed: false,
            reconciled: false,
        };
        if incident.is_none() && active_keys > 0 {
            let id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO incidents (id, network_id, basis, opened_ledger, status) VALUES ($1, $2, 'freeze_set_episode', $3, 'active')",
            )
            .bind(id)
            .bind(network)
            .bind(obs.ledger)
            .execute(&mut *tx)
            .await?;
            incident = Some(id);
            outcome.incident_opened = true;
        }
        if let Some(incident_id) = incident {
            let changed = has_changes;
            for change in &obs.freeze_changes {
                insert_event(
                    &mut tx,
                    incident_id,
                    change.ledger_sequence.get() as i64,
                    "freeze_change",
                    change.evidence_ref.as_deref(),
                )
                .await?;
            }
            for change in &obs.bypass_changes {
                insert_event(
                    &mut tx,
                    incident_id,
                    change.ledger_sequence.get() as i64,
                    "bypass_change",
                    change.evidence_ref.as_deref(),
                )
                .await?;
            }
            if changed {
                let (accounts, trustlines): (i64, i64) = sqlx::query_as(
                    "SELECT COUNT(*) FILTER (WHERE k.key_kind='account'), COUNT(*) FILTER (WHERE k.key_kind='trustline') \
                     FROM current_frozen_keys c JOIN ledger_keys k ON k.network_id=c.network_id AND k.key_hash=c.key_hash WHERE c.network_id=$1",
                )
                .bind(network)
                .fetch_one(&mut *tx)
                .await?;
                sqlx::query(
                    "INSERT INTO impact_snapshots (incident_id, ledger_sequence, total_frozen_accounts, total_frozen_trustlines, total_bypassed_txs) VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(incident_id)
                .bind(obs.ledger)
                .bind(accounts)
                .bind(trustlines)
                .bind(active_bypasses)
                .execute(&mut *tx)
                .await?;
            }
            if active_keys == 0 {
                sqlx::query("UPDATE incidents SET status='resolved', closed_ledger=$2 WHERE id=$1")
                    .bind(incident_id)
                    .bind(obs.ledger)
                    .execute(&mut *tx)
                    .await?;
                outcome.incident_closed = true;
            }
        }

        sqlx::query(
            "INSERT INTO ingestion_checkpoints (network_id, stream, last_complete_ledger, updated_at) VALUES ($1, 'cap77_config', $2, NOW()) \
             ON CONFLICT (network_id, stream) DO UPDATE SET last_complete_ledger = GREATEST(ingestion_checkpoints.last_complete_ledger, EXCLUDED.last_complete_ledger), updated_at = NOW()",
        )
        .bind(network)
        .bind(obs.ledger)
        .execute(&mut *tx)
        .await?;

        upsert_observation(
            &mut tx,
            obs.network_id,
            obs.latest_network_ledger,
            obs.protocol_version,
        )
        .await?;

        let stored_keys: Vec<Vec<u8>> =
            sqlx::query_scalar("SELECT key_hash FROM current_frozen_keys WHERE network_id=$1")
                .bind(network)
                .fetch_all(&mut *tx)
                .await?;
        let stored_bypasses: Vec<String> =
            sqlx::query_scalar("SELECT tx_hash FROM current_bypasses WHERE network_id=$1")
                .bind(network)
                .fetch_all(&mut *tx)
                .await?;
        let stored_key_set: HashSet<Vec<u8>> = stored_keys.into_iter().collect();
        let observed_key_set: HashSet<Vec<u8>> = obs
            .observed_keys
            .iter()
            .map(|(id, _)| id.as_bytes().to_vec())
            .collect();
        let stored_bypass_set: HashSet<String> = stored_bypasses.into_iter().collect();
        let observed_bypass_set: HashSet<String> = obs
            .observed_bypasses
            .iter()
            .map(|b| b.to_string())
            .collect();
        if stored_key_set == observed_key_set && stored_bypass_set == observed_bypass_set {
            sqlx::query(
                "UPDATE network_observations SET last_reconciled_ledger=$2, last_reconciled_at=NOW() WHERE network_id=$1",
            )
            .bind(network)
            .bind(obs.ledger)
            .execute(&mut *tx)
            .await?;
            outcome.reconciled = true;
        }

        tx.commit().await?;
        Ok(outcome)
    }
}

async fn insert_event(
    tx: &mut Transaction<'_, Postgres>,
    incident_id: Uuid,
    ledger: i64,
    kind: &str,
    evidence_ref: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO incident_events (incident_id, ledger_sequence, kind, evidence_ref) VALUES ($1, $2, $3, $4)",
    )
    .bind(incident_id)
    .bind(ledger)
    .bind(kind)
    .bind(evidence_ref.unwrap_or("none"))
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn upsert_observation(
    tx: &mut Transaction<'_, Postgres>,
    network_id: NetworkId,
    latest_network_ledger: i64,
    protocol_version: Option<i32>,
) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO network_observations (network_id, latest_network_ledger, protocol_version, observed_at) VALUES ($1, $2, $3, NOW()) \
         ON CONFLICT (network_id) DO UPDATE SET latest_network_ledger = GREATEST(network_observations.latest_network_ledger, EXCLUDED.latest_network_ledger), \
         protocol_version = COALESCE(EXCLUDED.protocol_version, network_observations.protocol_version), observed_at = NOW()",
    )
    .bind(network_id.as_uuid())
    .bind(latest_network_ledger)
    .bind(protocol_version)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
