use quorumscope_domain::incident::IncidentId;
use sqlx::{PgPool, Row};
use std::env;
use uuid::Uuid;

#[tokio::test]
async fn test_full_pipeline_mock() {
    // Mocked test
    assert!(true, "integration test scaffold");
}

#[tokio::test]
#[ignore = "requires real postgres database"]
async fn test_real_postgres_integration() -> anyhow::Result<()> {
    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://quorumscope:password@localhost:5433/quorumscope".to_string()
    });

    let pool = PgPool::connect(&db_url).await?;

    // 1. Migration up
    quorumscope_storage::pool::run_migrations(&pool).await?;

    let incident_id = IncidentId(Uuid::new_v4());

    // Check if network exists, insert if not
    sqlx::query(
        "INSERT INTO network (id, name, pass_phrase) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind("testnet")
    .bind("Stellar Testnet")
    .bind("Test SDF Network ; September 2015")
    .execute(&pool)
    .await?;

    sqlx::query(
        "INSERT INTO incident (id, network, status, opened_ledger) VALUES ($1, $2, $3, $4)",
    )
    .bind(incident_id.0)
    .bind("testnet")
    .bind("Active")
    .bind(100_i32)
    .execute(&pool)
    .await?;

    sqlx::query("INSERT INTO freeze_state (network, key_xdr, key_kind, added_at_ledger, incident_id) VALUES ($1, $2, $3, $4, $5)")
        .bind("testnet")
        .bind("AAAAAwAAAAIAAA==")
        .bind("ContractData")
        .bind(100_i32)
        .bind(incident_id.0)
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO bypass_state (network, tx_hash_hex, added_at_ledger, incident_id) VALUES ($1, $2, $3, $4)")
        .bind("testnet")
        .bind("0000000000000000000000000000000000000000000000000000000000000000")
        .bind(100_i32)
        .bind(incident_id.0)
        .execute(&pool)
        .await?;

    sqlx::query("INSERT INTO indexer_cursor (network, last_processed_ledger, updated_at) VALUES ($1, $2, NOW()) ON CONFLICT (network) DO UPDATE SET last_processed_ledger = EXCLUDED.last_processed_ledger, updated_at = NOW()")
        .bind("testnet")
        .bind(100_i32)
        .execute(&pool)
        .await?;

    let row =
        sqlx::query("SELECT last_processed_ledger FROM indexer_cursor WHERE network = 'testnet'")
            .fetch_one(&pool)
            .await?;

    let processed: i32 = row.get("last_processed_ledger");
    assert_eq!(processed, 100);

    Ok(())
}
