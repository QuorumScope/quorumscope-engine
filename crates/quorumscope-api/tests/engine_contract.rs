//! Contract tests against a real PostgreSQL database. They run when
//! API_TEST_DATABASE_URL is set and are skipped otherwise.
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use quorumscope_api::{ApiConfig, app_with_config};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use stellar_xdr::{
    AccountId, Asset, LedgerKey, LedgerKeyAccount, Memo, MuxedAccount, Operation, OperationBody,
    PaymentOp, Preconditions, PublicKey, SequenceNumber, Transaction, TransactionEnvelope,
    TransactionExt, TransactionV1Envelope, Uint256, VecM, WriteXdr,
};
use tower::ServiceExt;
use uuid::Uuid;

const PASSPHRASE: &str = "Test SDF Network ; September 2015";

fn account_key(byte: u8) -> LedgerKey {
    LedgerKey::Account(LedgerKeyAccount {
        account_id: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([byte; 32]))),
    })
}

fn key_xdr(byte: u8) -> Vec<u8> {
    account_key(byte)
        .to_xdr(stellar_xdr::Limits::none())
        .unwrap()
}

fn key_hash(byte: u8) -> Vec<u8> {
    Sha256::digest(key_xdr(byte)).to_vec()
}

fn payment_envelope(from: u8, to: u8) -> TransactionEnvelope {
    let op = Operation {
        source_account: None,
        body: OperationBody::Payment(PaymentOp {
            destination: MuxedAccount::Ed25519(Uint256([to; 32])),
            asset: Asset::Native,
            amount: 1,
        }),
    };
    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: Transaction {
            source_account: MuxedAccount::Ed25519(Uint256([from; 32])),
            fee: 100,
            seq_num: SequenceNumber(1),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: vec![op].try_into().unwrap(),
            ext: TransactionExt::V0,
        },
        signatures: VecM::default(),
    })
}

fn xdr_b64(env: &TransactionEnvelope) -> String {
    env.to_xdr_base64(stellar_xdr::Limits::none()).unwrap()
}

struct Fixture {
    pool: PgPool,
    network: Uuid,
}

impl Fixture {
    async fn connect() -> Option<PgPool> {
        let url = std::env::var("API_TEST_DATABASE_URL").ok()?;
        let pool = PgPool::connect(&url).await.unwrap();
        sqlx::migrate!("../../migrations").run(&pool).await.unwrap();
        Some(pool)
    }

    async fn network(pool: &PgPool, indexed: bool, protocol: i32) -> Self {
        let network = Uuid::new_v4();
        sqlx::query("INSERT INTO networks (id, name, passphrase) VALUES ($1, $2, $3)")
            .bind(network)
            .bind(format!("contract-{network}"))
            .bind(PASSPHRASE)
            .execute(pool)
            .await
            .unwrap();
        if indexed {
            sqlx::query("INSERT INTO ingestion_checkpoints (network_id, stream, last_complete_ledger, updated_at) VALUES ($1, 'cap77_config', 500, NOW())")
                .bind(network).execute(pool).await.unwrap();
            sqlx::query("INSERT INTO network_observations (network_id, latest_network_ledger, protocol_version, observed_at, last_reconciled_ledger, last_reconciled_at) VALUES ($1, 502, $2, NOW(), 500, NOW())")
                .bind(network).bind(protocol).execute(pool).await.unwrap();
        }
        Self {
            pool: pool.clone(),
            network,
        }
    }

    /// Key 2 is frozen and was frozen, unfrozen, then frozen again. Key 3 was unfrozen.
    async fn freeze_history(&self) {
        let pool = &self.pool;
        for byte in [2u8, 3u8] {
            sqlx::query("INSERT INTO ledger_keys (network_id, key_hash, canonical_key_xdr, key_kind, decoded_json, first_observed_ledger, last_observed_ledger) VALUES ($1, $2, $3, 'account', $4, 100, 500)")
                .bind(self.network).bind(key_hash(byte)).bind(key_xdr(byte))
                .bind(json!({"account_id": format!("synthetic-{byte}")}))
                .execute(pool).await.unwrap();
        }
        for (byte, ledger, action) in [
            (2u8, 100i64, "freeze"),
            (2, 200, "unfreeze"),
            (2, 300, "freeze"),
            (3, 100, "freeze"),
            (3, 150, "unfreeze"),
        ] {
            sqlx::query("INSERT INTO freeze_changes (id, network_id, ledger_sequence, key_hash, action, result, evidence_ref, created_at) VALUES ($1, $2, $3, $4, $5, 'changed', $6, NOW())")
                .bind(Uuid::new_v4()).bind(self.network).bind(ledger).bind(key_hash(byte))
                .bind(action).bind(format!("config_snapshot:{ledger}"))
                .execute(pool).await.unwrap();
        }
        sqlx::query("INSERT INTO current_frozen_keys (network_id, key_hash, active_since, last_changed, evidence_ref) VALUES ($1, $2, 300, 300, 'config_snapshot:300')")
            .bind(self.network).bind(key_hash(2)).execute(pool).await.unwrap();
        let incident = Uuid::new_v4();
        sqlx::query("INSERT INTO incidents (id, network_id, basis, opened_ledger, status) VALUES ($1, $2, 'freeze_set_episode', 300, 'active')")
            .bind(incident).bind(self.network).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO impact_snapshots (incident_id, ledger_sequence, total_frozen_accounts, total_frozen_trustlines, total_bypassed_txs) VALUES ($1, 300, 1, 0, 0)")
            .bind(incident).execute(pool).await.unwrap();
    }
}

async fn call(pool: &PgPool, config: ApiConfig, req: Request<Body>) -> (StatusCode, String, Value) {
    let response = app_with_config(pool.clone(), config)
        .oneshot(req)
        .await
        .unwrap();
    let status = response.status();
    let request_id = response.headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_string();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, request_id, serde_json::from_slice(&bytes).unwrap())
}

async fn get(pool: &PgPool, path: &str) -> (StatusCode, String, Value) {
    let req = Request::builder().uri(path).body(Body::empty()).unwrap();
    call(pool, ApiConfig::default(), req).await
}

async fn preflight(pool: &PgPool, network: Uuid, xdr: &str) -> (StatusCode, String, Value) {
    let body = json!({ "transaction_xdr": xdr, "network_id": network }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/v1/preflight")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    call(pool, ApiConfig::default(), req).await
}

#[tokio::test]
async fn preflight_reports_clear_blocked_and_bypassed() {
    let Some(pool) = Fixture::connect().await else {
        return;
    };
    let fx = Fixture::network(&pool, true, 25).await;
    fx.freeze_history().await;

    let (status, request_id, body) =
        preflight(&pool, fx.network, &xdr_b64(&payment_envelope(8, 9))).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "clear");
    assert_eq!(body["confidence"], "deterministic");
    assert_eq!(body["source_ledger"], 500);
    assert_eq!(body["freshness"]["status"], "current");
    assert_eq!(body["request_id"], request_id);
    assert!(body["findings"].as_array().unwrap().is_empty());
    assert_eq!(body["transaction_hash"].as_str().unwrap().len(), 64);

    let blocked_env = payment_envelope(8, 2);
    let (_, _, body) = preflight(&pool, fx.network, &xdr_b64(&blocked_env)).await;
    assert_eq!(body["status"], "blocked_validation");
    assert_eq!(body["confidence"], "deterministic");
    let finding = &body["findings"][0];
    assert_eq!(finding["implicated_keys"][0], hex::encode(key_hash(2)));
    assert_eq!(
        finding["protocol_path"],
        "operation 0 (Payment): destination account"
    );

    // Line breaks in pasted input are ignored.
    let wrapped = xdr_b64(&blocked_env)
        .as_bytes()
        .chunks(40)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let (_, _, body) = preflight(&pool, fx.network, &wrapped).await;
    assert_eq!(body["status"], "blocked_validation");

    let hash = body["transaction_hash"].as_str().unwrap().to_string();
    sqlx::query("INSERT INTO current_bypasses (network_id, tx_hash, active_since, last_changed) VALUES ($1, $2, 310, 310)")
        .bind(fx.network).bind(&hash).execute(&pool).await.unwrap();
    let (_, _, body) = preflight(&pool, fx.network, &xdr_b64(&blocked_env)).await;
    assert_eq!(body["status"], "allowed_by_bypass");
    assert_eq!(body["is_bypassed"], true);
    assert!(!body["findings"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn preflight_rejects_bad_input_and_never_claims_clear_without_state() {
    let Some(pool) = Fixture::connect().await else {
        return;
    };
    let fresh = Fixture::network(&pool, true, 25).await;
    for bad in ["", "   ", "not base64 !!", "AAAA"] {
        let (status, _, body) = preflight(&pool, fresh.network, bad).await;
        assert_eq!(status, StatusCode::OK, "{bad:?}");
        assert_eq!(body["status"], "invalid_input", "{bad:?}");
        assert!(body["transaction_hash"].is_null());
    }

    let valid = xdr_b64(&payment_envelope(8, 9));
    let unindexed = Fixture::network(&pool, false, 25).await;
    let (_, _, body) = preflight(&pool, unindexed.network, &valid).await;
    assert_eq!(body["status"], "state_unavailable");
    assert_eq!(body["confidence"], "insufficient_information");
    assert!(body["source_ledger"].is_null());
    assert_eq!(body["freshness"]["status"], "unknown");

    sqlx::query("UPDATE network_observations SET observed_at = NOW() - INTERVAL '1 hour' WHERE network_id=$1")
        .bind(fresh.network).execute(&pool).await.unwrap();
    let (_, _, body) = preflight(&pool, fresh.network, &valid).await;
    assert_eq!(body["status"], "state_unavailable");
    assert_eq!(body["freshness"]["status"], "stale");
}

#[tokio::test]
async fn preflight_body_errors_use_the_error_envelope() {
    let Some(pool) = Fixture::connect().await else {
        return;
    };
    let post = |body: String| {
        Request::builder()
            .method("POST")
            .uri("/api/v1/preflight")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap()
    };
    let (status, request_id, body) =
        call(&pool, ApiConfig::default(), post("{not json".into())).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "invalid_input");
    assert_eq!(body["error"]["request_id"], request_id);
    assert_eq!(body["request_id"], request_id);
    assert!(body["error"]["details"].is_object());

    let huge = json!({ "transaction_xdr": "A".repeat(300 * 1024) }).to_string();
    let (status, _, body) = call(&pool, ApiConfig::default(), post(huge)).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(body["error"]["code"], "payload_too_large");
}

#[tokio::test]
async fn impact_returns_stored_evidence_and_supports_real_filters() {
    let Some(pool) = Fixture::connect().await else {
        return;
    };
    let fx = Fixture::network(&pool, true, 25).await;
    fx.freeze_history().await;
    let base = format!("/api/v1/impact?network_id={}", fx.network);

    let (status, _, body) = get(&pool, &base).await;
    assert_eq!(status, StatusCode::OK);
    let items = body["items"].as_array().unwrap();
    let classes: Vec<_> = items
        .iter()
        .map(|i| i["evidence_class"].as_str().unwrap())
        .collect();
    assert_eq!(classes, ["direct", "protocol_derived"]);
    assert_eq!(items[0]["key_id"], hex::encode(key_hash(2)));
    assert_eq!(items[0]["key_kind"], "account");
    assert_eq!(items[0]["observation_window"]["first_ledger"], 300);
    assert_eq!(items[0]["observation_window"]["last_ledger"], 500);
    assert_eq!(
        items[0]["observation_window"]["is_complete_for_range"],
        false
    );
    assert_eq!(items[0]["evidence_ref"], "config_snapshot:300");
    assert_eq!(items[1]["details"]["frozen_accounts"], 1);
    assert_eq!(
        body["uncollected_evidence_classes"],
        json!(["recently_observed", "dependency_observed", "inferred"])
    );

    let (_, _, body) = get(&pool, &format!("{base}&evidence_class=direct")).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    let (_, _, body) = get(&pool, &format!("{base}&evidence_class=inferred")).await;
    assert!(
        body["items"].as_array().unwrap().is_empty(),
        "no inferred evidence is stored"
    );
    let (_, _, body) = get(&pool, &format!("{base}&key_kind=trustline")).await;
    assert!(body["items"].as_array().unwrap().is_empty());
    let (_, _, body) = get(
        &pool,
        &format!("{base}&key_id={}", hex::encode(key_hash(2))),
    )
    .await;
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    let (_, _, body) = get(&pool, &format!("{base}&page_size=1&page=2")).await;
    assert_eq!(body["items"][0]["evidence_class"], "protocol_derived");

    for bad in [
        "evidence_class=nope",
        "key_kind=nope",
        "key_id=zz",
        "page_size=0",
    ] {
        let (status, request_id, body) = get(&pool, &format!("{base}&{bad}")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(body["error"]["request_id"], request_id);
    }
}

#[tokio::test]
async fn key_detail_includes_decoded_content_and_full_history() {
    let Some(pool) = Fixture::connect().await else {
        return;
    };
    let fx = Fixture::network(&pool, true, 25).await;
    fx.freeze_history().await;
    let scope = format!("network_id={}", fx.network);

    let (status, _, body) = get(
        &pool,
        &format!("/api/v1/frozen-keys/{}?{scope}", hex::encode(key_hash(2))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["active"], true);
    assert_eq!(body["kind"], "account");
    assert_eq!(body["decoded"]["account_id"], "synthetic-2");
    assert_eq!(body["first_frozen_ledger"], 100);
    assert_eq!(body["active_since"], 300);
    let history = body["history"].as_array().unwrap();
    let actions: Vec<_> = history
        .iter()
        .map(|h| h["action"].as_str().unwrap())
        .collect();
    assert_eq!(actions, ["freeze", "unfreeze", "freeze"]);
    assert_eq!(history[1]["ledger_sequence"], 200);
    assert_eq!(history[1]["evidence_ref"], "config_snapshot:200");

    let (status, _, body) = get(
        &pool,
        &format!("/api/v1/frozen-keys/{}?{scope}", hex::encode(key_hash(3))),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["active"], false);
    assert!(body["active_since"].is_null());
    assert_eq!(body["history"].as_array().unwrap().len(), 2);

    let (status, _, _) = get(
        &pool,
        &format!("/api/v1/frozen-keys/{}?{scope}", hex::encode([0u8; 32])),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let (_, _, body) = get(&pool, &format!("/api/v1/frozen-keys?{scope}")).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 1);
    let (_, _, body) = get(&pool, &format!("/api/v1/frozen-keys?{scope}&active=false")).await;
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    let (_, _, body) = get(
        &pool,
        &format!("/api/v1/frozen-keys?{scope}&kind=contract_code"),
    )
    .await;
    assert!(body["items"].as_array().unwrap().is_empty());
    let (status, _, _) = get(&pool, &format!("/api/v1/frozen-keys?{scope}&kind=bogus")).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn state_endpoints_report_freshness_and_compatibility() {
    let Some(pool) = Fixture::connect().await else {
        return;
    };
    let fx = Fixture::network(&pool, true, 26).await;
    let config = ApiConfig {
        verified_protocol_max: Some(25),
        ..ApiConfig::default()
    };
    for path in ["network", "freeze-state", "status"] {
        let req = Request::builder()
            .uri(format!("/api/v1/{path}?network_id={}", fx.network))
            .body(Body::empty())
            .unwrap();
        let (status, _, body) = call(&pool, config.clone(), req).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        let f = &body["freshness"];
        assert_eq!(f["status"], "current", "{path}");
        assert_eq!(f["source_ledger"], 500);
        assert_eq!(f["latest_indexed_ledger"], 500);
        assert_eq!(f["latest_network_ledger"], 502);
        assert_eq!(f["ingestion_lag_ledgers"], 2);
        assert_eq!(f["current_protocol_version"], 26);
        assert_eq!(f["verified_protocol_max"], 25);
        assert_eq!(f["compatibility"], "unverified_protocol");
        assert_eq!(f["last_reconciled_ledger"], 500);
        assert!(f["last_reconciled_at"].is_string());
    }

    let unindexed = Fixture::network(&pool, false, 25).await;
    let (status, _, body) = get(
        &pool,
        &format!("/api/v1/network?network_id={}", unindexed.network),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["freshness"]["status"], "unknown");
    assert_eq!(body["freshness"]["compatibility"], "unknown");
}

#[tokio::test]
async fn openapi_lists_every_implemented_route() {
    let Some(pool) = Fixture::connect().await else {
        return;
    };
    let (_, _, body) = get(&pool, "/openapi.json").await;
    for path in [
        "/api/v1/preflight",
        "/api/v1/impact",
        "/api/v1/network",
        "/api/v1/freeze-state",
        "/api/v1/frozen-keys",
        "/api/v1/frozen-keys/{id}",
        "/api/v1/bypasses",
        "/api/v1/incidents",
        "/api/v1/incidents/{id}",
        "/api/v1/incidents/{id}/timeline",
        "/api/v1/status",
        "/health/live",
        "/health/ready",
    ] {
        assert!(body["paths"][path].is_object(), "{path}");
    }
    assert!(body["paths"]["/api/v1/preflight"]["post"].is_object());
}
