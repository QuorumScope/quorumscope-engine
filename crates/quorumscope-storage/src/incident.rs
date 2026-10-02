use crate::error::StorageError;
use chrono::{DateTime, TimeZone, Utc};
use quorumscope_domain::incident::Incident;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub struct IncidentRow {
    pub id: Uuid,
    pub network_id: Uuid,
    pub basis: String,
    pub opened_ledger: i64,
    pub opened_close_time: Option<DateTime<Utc>>,
    pub closed_ledger: Option<i64>,
    pub closed_close_time: Option<DateTime<Utc>>,
    pub status: String,
}

pub struct IncidentRepository {
    pool: PgPool,
}

impl IncidentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_incident(&self, incident: &Incident) -> Result<(), StorageError> {
        let status = incident.status.to_string();
        let network_uuid = incident.network_id.as_uuid();

        let open_time = incident
            .opened_close_time
            .map(|t| Utc.timestamp_opt(t.as_timestamp(), 0).unwrap());
        let close_time = incident
            .closed_close_time
            .map(|t| Utc.timestamp_opt(t.as_timestamp(), 0).unwrap());

        sqlx::query(
            r#"
            INSERT INTO incidents (id, network_id, basis, opened_ledger, opened_close_time, closed_ledger, closed_close_time, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#)
            .bind(incident.id.as_uuid())
            .bind(network_uuid)
            .bind(&incident.basis)
            .bind(incident.opened_ledger.get() as i64)
            .bind(open_time)
            .bind(incident.closed_ledger.map(|l| l.get() as i64))
            .bind(close_time)
            .bind(status)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
