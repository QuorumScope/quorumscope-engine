//! Mocked RPC test. The RPC server is simulated with wiremock and the database is real
//! PostgreSQL. It runs when API_TEST_DATABASE_URL is set. It verifies the indexer's handling of
//! a non-empty freeze set and bypass list, which the live testnet could not exercise.
//! Keys and hashes are synthetic.
use quorumscope_indexer::sync::SyncTask;
use quorumscope_rpc::client::RpcClient;
use quorumscope_rpc::config::RpcConfig;
use quorumscope_storage::indexer::IndexerRepository;
use quorumscope_storage::network::ensure_network;
use quorumscope_xdr::codec::encode_base64;
use sqlx::PgPool;
use std::time::Duration;
use stellar_xdr::{
    AccountId, ConfigSettingEntry, EncodedLedgerKey, FreezeBypassTxs, FrozenLedgerKeys, Hash,
    LedgerEntryData, LedgerKey, LedgerKeyAccount, Limits, PublicKey, Uint256, VecM, WriteXdr,
};
use uuid::Uuid;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn entry_xdr(setting: ConfigSettingEntry) -> String {
    encode_base64(
        &LedgerEntryData::ConfigSetting(setting)
            .to_xdr(Limits::none())
            .unwrap(),
    )
}

fn entries_response(
    ledger: u32,
    keys: Vec<LedgerKey>,
    bypasses: Vec<[u8; 32]>,
) -> serde_json::Value {
    let encoded: Vec<EncodedLedgerKey> = keys
        .iter()
        .map(|k| EncodedLedgerKey(k.to_xdr(Limits::none()).unwrap().try_into().unwrap()))
        .collect();
    let frozen = entry_xdr(ConfigSettingEntry::FrozenLedgerKeys(FrozenLedgerKeys {
        keys: encoded.try_into().unwrap(),
    }));
    let hashes: Vec<Hash> = bypasses.into_iter().map(Hash).collect();
    let bypass = entry_xdr(ConfigSettingEntry::FreezeBypassTxs(FreezeBypassTxs {
        tx_hashes: VecM::try_from(hashes).unwrap(),
    }));
    serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "result": { "latestLedger": ledger, "entries": [
            { "key": "k1", "xdr": frozen, "lastModifiedLedgerSeq": ledger },
            { "key": "k2", "xdr": bypass, "lastModifiedLedgerSeq": ledger }
        ]}
    })
}

fn account_key(byte: u8) -> LedgerKey {
    LedgerKey::Account(LedgerKeyAccount {
        account_id: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([byte; 32]))),
    })
}

#[tokio::test]
async fn indexer_records_freeze_bypass_episode_and_unfreeze_from_rpc_responses() {
    let Ok(url) = std::env::var("API_TEST_DATABASE_URL") else {
        return;
    };
    let pool = PgPool::connect(&url).await.unwrap();
    quorumscope_storage::pool::run_migrations(&pool)
        .await
        .unwrap();
    let network = ensure_network(
        &pool,
        &format!("mocked-{}", Uuid::new_v4()),
        "mock passphrase",
    )
    .await
    .unwrap();

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(body_partial_json(
            serde_json::json!({ "method": "getLatestLedger" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0", "id": 2,
            "result": { "id": "x", "protocolVersion": 28, "sequence": 120 }
        })))
        .mount(&server)
        .await;
    // First poll sees one frozen key and one bypass. Later polls see an empty set.
    Mock::given(method("POST"))
        .and(body_partial_json(
            serde_json::json!({ "method": "getLedgerEntries" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(entries_response(
            100,
            vec![account_key(5)],
            vec![[9; 32]],
        )))
        .up_to_n_times(2)
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(body_partial_json(
            serde_json::json!({ "method": "getLedgerEntries" }),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(entries_response(
            110,
            vec![],
            vec![],
        )))
        .with_priority(2)
        .mount(&server)
        .await;

    let client = RpcClient::new(RpcConfig {
        endpoint: server.uri().parse().unwrap(),
        timeout: Duration::from_secs(5),
        max_retries: 0,
        base_backoff: Duration::from_millis(10),
    })
    .unwrap();
    let task = SyncTask::new(client, IndexerRepository::new(pool.clone()), network);
    let scalar = |sql: &'static str| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>(sql)
                .bind(network.as_uuid())
                .fetch_one(&pool)
                .await
                .unwrap()
        }
    };

    task.run_once().await.unwrap();
    assert_eq!(
        scalar("SELECT COUNT(*) FROM current_frozen_keys WHERE network_id=$1").await,
        1
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM current_bypasses WHERE network_id=$1").await,
        1
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM ledger_keys WHERE network_id=$1 AND key_kind='account'").await,
        1
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM incidents WHERE network_id=$1 AND status='active'").await,
        1
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM freeze_changes WHERE network_id=$1 AND evidence_ref LIKE 'config_snapshot:%'").await,
        1
    );

    // A second identical poll adds no history.
    task.run_once().await.unwrap();
    assert_eq!(
        scalar("SELECT COUNT(*) FROM freeze_changes WHERE network_id=$1").await,
        1
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM bypass_changes WHERE network_id=$1").await,
        1
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM config_snapshots WHERE network_id=$1").await,
        2
    );

    // The set empties: the key is unfrozen, the bypass removed, and the episode resolves.
    task.run_once().await.unwrap();
    assert_eq!(
        scalar("SELECT COUNT(*) FROM current_frozen_keys WHERE network_id=$1").await,
        0
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM current_bypasses WHERE network_id=$1").await,
        0
    );
    assert_eq!(
        scalar("SELECT COUNT(*) FROM freeze_changes WHERE network_id=$1 AND action='unfreeze'")
            .await,
        1
    );
    assert_eq!(scalar("SELECT COUNT(*) FROM incidents WHERE network_id=$1 AND status='resolved' AND closed_ledger=110").await, 1);
    assert_eq!(
        scalar("SELECT last_reconciled_ledger FROM network_observations WHERE network_id=$1").await,
        110
    );
}
