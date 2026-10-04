//! Runs against a real PostgreSQL database when API_TEST_DATABASE_URL is set.
use quorumscope_domain::freeze::{
    DecodedFrozenKey, FreezeAction, FreezeChange, FreezeChangeResult, FrozenKeyId, FrozenKeyKind,
};
use quorumscope_domain::ledger::LedgerSequence;
use quorumscope_storage::network::ensure_network;
use quorumscope_storage::poll::{PollObservation, PollRepository, SnapshotRecord};
use std::time::SystemTime;
use uuid::Uuid;

fn key(byte: u8) -> (FrozenKeyId, DecodedFrozenKey) {
    (
        FrozenKeyId::new([byte; 32]),
        DecodedFrozenKey {
            kind: FrozenKeyKind::Account,
            decoded_json: format!("{{\"account_id\":\"test-{byte}\"}}"),
            canonical_xdr: vec![byte; 8],
        },
    )
}

fn change(
    network: quorumscope_domain::network::NetworkId,
    ledger: u32,
    id: FrozenKeyId,
    action: FreezeAction,
    evidence: &str,
) -> FreezeChange {
    FreezeChange {
        network_id: network,
        ledger_sequence: LedgerSequence::new(ledger).unwrap(),
        key_hash: id,
        action,
        result: FreezeChangeResult::Changed,
        evidence_ref: Some(evidence.to_string()),
        created_at: SystemTime::now(),
    }
}

#[tokio::test]
async fn poll_opens_and_closes_an_episode_with_evidence() {
    let Ok(url) = std::env::var("API_TEST_DATABASE_URL") else {
        return;
    };
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    quorumscope_storage::pool::run_migrations(&pool)
        .await
        .unwrap();

    let name = format!("poll-test-{}", Uuid::new_v4());
    let network = ensure_network(&pool, &name, "poll passphrase")
        .await
        .unwrap();
    assert_eq!(
        network,
        ensure_network(&pool, &name, "poll passphrase")
            .await
            .unwrap()
    );
    assert!(
        ensure_network(&pool, &name, "other passphrase")
            .await
            .is_err()
    );

    let repo = PollRepository::new(pool.clone());
    let (id, decoded) = key(7);
    let snapshot = SnapshotRecord {
        id: Uuid::new_v4(),
        setting_id: "FrozenLedgerKeys",
        raw_xdr: vec![1, 2, 3],
        parsed_json: serde_json::json!({ "keys_count": 1 }),
    };
    let evidence = snapshot.evidence_ref();
    let opened = repo
        .apply(&PollObservation {
            network_id: network,
            ledger: 100,
            latest_network_ledger: 101,
            protocol_version: Some(25),
            snapshots: vec![snapshot],
            observed_keys: vec![(id, decoded)],
            observed_bypasses: vec![],
            freeze_changes: vec![change(network, 100, id, FreezeAction::Freeze, &evidence)],
            bypass_changes: vec![],
        })
        .await
        .unwrap();
    assert!(opened.incident_opened && opened.reconciled && !opened.incident_closed);

    let (kind, active_since, stored_evidence): (String, i64, Option<String>) = sqlx::query_as(
        "SELECT k.key_kind, c.active_since, c.evidence_ref FROM current_frozen_keys c JOIN ledger_keys k USING (network_id, key_hash) WHERE c.network_id=$1",
    )
    .bind(network.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((kind.as_str(), active_since), ("account", 100));
    assert_eq!(stored_evidence.as_deref(), Some(evidence.as_str()));
    let (events, snapshots): (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM incident_events e JOIN incidents i ON i.id=e.incident_id WHERE i.network_id=$1), \
                (SELECT COUNT(*) FROM impact_snapshots s JOIN incidents i ON i.id=s.incident_id WHERE i.network_id=$1)",
    )
    .bind(network.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((events, snapshots), (1, 1));

    let repeat_snapshot = SnapshotRecord {
        id: Uuid::new_v4(),
        setting_id: "FrozenLedgerKeys",
        raw_xdr: vec![1, 2, 3],
        parsed_json: serde_json::json!({ "keys_count": 1 }),
    };
    let unchanged = repo
        .apply(&PollObservation {
            network_id: network,
            ledger: 100,
            latest_network_ledger: 101,
            protocol_version: Some(25),
            snapshots: vec![repeat_snapshot],
            observed_keys: vec![key(7)],
            observed_bypasses: vec![],
            freeze_changes: vec![],
            bypass_changes: vec![],
        })
        .await
        .unwrap();
    assert!(!unchanged.incident_opened && unchanged.reconciled);
    let (changes,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM freeze_changes WHERE network_id=$1")
            .bind(network.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(changes, 1, "repeating a poll must not duplicate history");
    let (stored_snapshots,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM config_snapshots WHERE network_id=$1")
            .bind(network.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored_snapshots, 1, "an unchanged entry is stored once");

    let closed = repo
        .apply(&PollObservation {
            network_id: network,
            ledger: 105,
            latest_network_ledger: 105,
            protocol_version: Some(25),
            snapshots: vec![],
            observed_keys: vec![],
            observed_bypasses: vec![],
            freeze_changes: vec![change(network, 105, id, FreezeAction::Unfreeze, "none")],
            bypass_changes: vec![],
        })
        .await
        .unwrap();
    assert!(closed.incident_closed && closed.reconciled);
    let (status, closed_ledger): (String, Option<i64>) =
        sqlx::query_as("SELECT status, closed_ledger FROM incidents WHERE network_id=$1")
            .bind(network.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((status.as_str(), closed_ledger), ("resolved", Some(105)));
}
