use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use crate::error::StorageError;

#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
}

impl StorageConfig {
    pub fn new(database_url: String) -> Self {
        Self {
            database_url,
            max_connections: 10,
            connect_timeout: Duration::from_secs(10),
        }
    }
}

pub async fn connect(config: &StorageConfig) -> Result<PgPool, StorageError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .acquire_timeout(config.connect_timeout)
        .connect(&config.database_url)
        .await?;
        
    Ok(pool)
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), StorageError> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}
