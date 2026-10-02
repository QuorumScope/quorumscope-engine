use crate::error::StorageError;
use chrono::Utc;
use sqlx::PgPool;

pub struct MetricsRepository {
    pool: PgPool,
}

impl MetricsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_metric(
        &self,
        name: &str,
        value: f64,
        labels: serde_json::Value,
    ) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT INTO metrics (name, value, labels, recorded_at)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(name)
        .bind(value)
        .bind(labels)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
