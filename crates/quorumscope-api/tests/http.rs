use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

fn test_app() -> axum::Router {
    let pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(100))
        .connect_lazy("postgres://localhost:1/quorumscope")
        .unwrap();
    quorumscope_api::app(pool)
}

async fn response(path: &str) -> (StatusCode, Uuid, Value) {
    let response = test_app()
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let request_id = Uuid::parse_str(
        response
            .headers()
            .get("x-request-id")
            .unwrap()
            .to_str()
            .unwrap(),
    )
    .unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (status, request_id, serde_json::from_slice(&body).unwrap())
}

#[tokio::test]
async fn live_and_openapi_succeed() {
    let (status, _, body) = response("/health/live").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "live");

    let (status, _, body) = response("/openapi.json").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["paths"]["/api/v1/incidents/{id}/timeline"].is_object());
}

#[tokio::test]
async fn unknown_route_has_not_found_envelope() {
    let (status, request_id, body) = response("/api/v1/unknown").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["request_id"], request_id.to_string());
}

#[tokio::test]
async fn unsupported_method_has_error_envelope() {
    let response = test_app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/network")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    let request_id = response
        .headers()
        .get("x-request-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(body["error"]["code"], "method_not_allowed");
    assert_eq!(body["request_id"], request_id);
}

#[tokio::test]
async fn malformed_id_and_pagination_are_invalid_input() {
    for path in [
        "/api/v1/frozen-keys/not-hex",
        "/api/v1/incidents/not-a-uuid",
        "/api/v1/frozen-keys?page=0",
        "/api/v1/network?network_id=bad",
        "/api/v1/frozen-keys?page=abc",
    ] {
        let (status, request_id, body) = response(path).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{path}");
        assert_eq!(body["error"]["code"], "invalid_input");
        assert_eq!(body["request_id"], request_id.to_string());
    }
}

#[tokio::test]
async fn storage_failure_is_reported_without_leaking_details() {
    let (status, request_id, body) = response("/api/v1/network").await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"]["code"], "storage_error");
    assert_eq!(body["request_id"], request_id.to_string());

    let (status, request_id, body) = response("/health/ready").await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["error"]["code"], "storage_unavailable");
    assert_eq!(body["request_id"], request_id.to_string());
}

#[tokio::test]
async fn postgres_reads_return_state_and_resource_not_found() {
    let Ok(database_url) = std::env::var("API_TEST_DATABASE_URL") else {
        return;
    };
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    sqlx::migrate!("../../migrations").run(&pool).await.unwrap();

    let network_id = Uuid::new_v4();
    let incident_id = Uuid::new_v4();
    let key = vec![0xabu8; 32];
    sqlx::query("INSERT INTO networks (id, name, passphrase) VALUES ($1, $2, $3)")
        .bind(network_id)
        .bind(format!("api-test-{network_id}"))
        .bind("test passphrase")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO ingestion_checkpoints (network_id, stream, last_complete_ledger, updated_at) VALUES ($1, 'cap77_config', 123, NOW())")
        .bind(network_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO current_frozen_keys (network_id, key_hash, active_since, last_changed) VALUES ($1, $2, 120, 123)")
        .bind(network_id).bind(&key).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO current_bypasses (network_id, tx_hash, active_since, last_changed) VALUES ($1, $2, 121, 123)")
        .bind(network_id).bind("cd".repeat(32)).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO incidents (id, network_id, basis, opened_ledger, status) VALUES ($1, $2, 'freeze_set_episode', 120, 'active')")
        .bind(incident_id).bind(network_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO incident_events (incident_id, ledger_sequence, kind, evidence_ref) VALUES ($1, 120, 'freeze_change', 'evidence-1')")
        .bind(incident_id).execute(&pool).await.unwrap();

    let app = quorumscope_api::app(pool.clone());
    async fn get(app: &axum::Router, path: String) -> (StatusCode, Value) {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }
    let scope = format!("network_id={network_id}");
    let (code, body) = get(&app, format!("/api/v1/network?{scope}")).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["id"], network_id.to_string());
    let (code, body) = get(
        &app,
        format!("/api/v1/network?network_id={}", Uuid::new_v4()),
    )
    .await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");

    let (code, body) = get(&app, format!("/api/v1/freeze-state?{scope}")).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["latest_ledger"], 123);
    assert_eq!(body["frozen_key_count"], 1);
    assert_eq!(body["bypass_count"], 1);
    assert_eq!(body["active_incident_count"], 1);

    let (code, body) = get(&app, format!("/api/v1/frozen-keys?{scope}&page_size=1")).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["items"][0]["id"], hex::encode(&key));
    let (code, body) = get(
        &app,
        format!("/api/v1/frozen-keys/{}?{scope}", hex::encode(&key)),
    )
    .await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["active_since"], 120);
    let (code, body) = get(
        &app,
        format!("/api/v1/frozen-keys/{}?{scope}", "ef".repeat(32)),
    )
    .await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");

    let (code, body) = get(&app, format!("/api/v1/bypasses?{scope}")).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["items"][0]["tx_hash"], "cd".repeat(32));

    let (code, body) = get(&app, format!("/api/v1/incidents?{scope}")).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["items"][0]["id"], incident_id.to_string());
    let (code, body) = get(&app, format!("/api/v1/incidents/{incident_id}?{scope}")).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["opened_ledger"], 120);
    let (code, body) = get(
        &app,
        format!("/api/v1/incidents/{}?{scope}", Uuid::new_v4()),
    )
    .await;
    assert_eq!(code, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");
    let (code, body) = get(
        &app,
        format!("/api/v1/incidents/{incident_id}/timeline?{scope}"),
    )
    .await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["items"][0]["evidence_ref"], "evidence-1");

    let (code, body) = get(&app, format!("/api/v1/status?{scope}")).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["last_complete_ledger"], 123);
    let (code, body) = get(&app, "/health/ready".to_string()).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(body["status"], "ready");

    sqlx::query("DELETE FROM incident_events WHERE incident_id=$1")
        .bind(incident_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM incidents WHERE id=$1")
        .bind(incident_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM current_frozen_keys WHERE network_id=$1")
        .bind(network_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM current_bypasses WHERE network_id=$1")
        .bind(network_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM ingestion_checkpoints WHERE network_id=$1")
        .bind(network_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM networks WHERE id=$1")
        .bind(network_id)
        .execute(&pool)
        .await
        .unwrap();
}
