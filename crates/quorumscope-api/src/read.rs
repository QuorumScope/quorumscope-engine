use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::error::{ApiError, RequestId};

#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ScopeQuery {
    /// Network UUID. When omitted, the first network by name is selected.
    pub network_id: Option<Uuid>,
}

#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListQuery {
    /// Network UUID. When omitted, the first network by name is selected.
    pub network_id: Option<Uuid>,
    /// One-based page number. Defaults to 1.
    pub page: Option<i64>,
    /// Items per page, from 1 to 100. Defaults to 50.
    pub page_size: Option<i64>,
}

impl ListQuery {
    fn pagination(&self, id: RequestId) -> Result<(i64, i64), ApiError> {
        let page = self.page.unwrap_or(1);
        let size = self.page_size.unwrap_or(50);
        if page < 1
            || !(1..=100).contains(&size)
            || page
                .checked_sub(1)
                .and_then(|p| p.checked_mul(size))
                .is_none()
        {
            return Err(ApiError::bad_request(id, "invalid pagination"));
        }
        Ok((page, size))
    }
}

#[derive(Clone, Serialize, ToSchema, FromRow)]
pub struct NetworkResponse {
    pub id: Uuid,
    pub name: String,
    pub passphrase: String,
}

async fn selected_network(
    pool: &PgPool,
    id: RequestId,
    network_id: Option<Uuid>,
) -> Result<NetworkResponse, ApiError> {
    sqlx::query_as::<_, NetworkResponse>("SELECT id, name, passphrase FROM networks WHERE ($1::uuid IS NULL OR id = $1) ORDER BY name LIMIT 1")
        .bind(network_id).fetch_optional(pool).await.map_err(|e| ApiError::storage(id, e))?
        .ok_or_else(|| ApiError::not_found(id, "Network not found"))
}

#[derive(Serialize, ToSchema)]
pub struct FreezeStateResponse {
    pub network_id: Uuid,
    pub latest_ledger: Option<i64>,
    pub frozen_key_count: i64,
    pub bypass_count: i64,
    pub active_incident_count: i64,
}

#[derive(Serialize, ToSchema)]
pub struct FrozenKeyResponse {
    pub id: String,
    pub network_id: Uuid,
    pub active_since: i64,
    pub last_changed: i64,
    pub evidence_ref: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct BypassResponse {
    pub tx_hash: String,
    pub network_id: Uuid,
    pub active_since: i64,
    pub last_changed: i64,
    pub evidence_ref: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct IncidentResponse {
    pub id: Uuid,
    pub network_id: Uuid,
    pub basis: String,
    pub opened_ledger: i64,
    pub opened_close_time: Option<String>,
    pub closed_ledger: Option<i64>,
    pub closed_close_time: Option<String>,
    pub status: String,
}

#[derive(Serialize, ToSchema)]
pub struct TimelineEventResponse {
    pub id: i32,
    pub incident_id: Uuid,
    pub ledger_sequence: i64,
    pub kind: String,
    pub evidence_ref: String,
    pub created_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct FrozenKeysResponse {
    pub items: Vec<FrozenKeyResponse>,
    pub page: i64,
    pub page_size: i64,
}
#[derive(Serialize, ToSchema)]
pub struct BypassesResponse {
    pub items: Vec<BypassResponse>,
    pub page: i64,
    pub page_size: i64,
}
#[derive(Serialize, ToSchema)]
pub struct IncidentsResponse {
    pub items: Vec<IncidentResponse>,
    pub page: i64,
    pub page_size: i64,
}
#[derive(Serialize, ToSchema)]
pub struct TimelineResponse {
    pub items: Vec<TimelineEventResponse>,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Serialize, ToSchema)]
pub struct StatusResponse {
    pub network_id: Uuid,
    pub stream: String,
    pub last_complete_ledger: i64,
    pub updated_at: String,
}

#[utoipa::path(get, path = "/api/v1/network", params(ScopeQuery), responses((status = 200, body = NetworkResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn network(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<NetworkResponse>, ApiError> {
    Ok(Json(selected_network(&pool, id, q.network_id).await?))
}

#[utoipa::path(get, path = "/api/v1/freeze-state", params(ScopeQuery), responses((status = 200, body = FreezeStateResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn freeze_state(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<FreezeStateResponse>, ApiError> {
    let network = selected_network(&pool, id, q.network_id).await?;
    let row: (Option<i64>, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT MAX(last_complete_ledger) FROM ingestion_checkpoints WHERE network_id=$1), \
         (SELECT COUNT(*) FROM current_frozen_keys WHERE network_id=$1), \
         (SELECT COUNT(*) FROM current_bypasses WHERE network_id=$1), \
         (SELECT COUNT(*) FROM incidents WHERE network_id=$1 AND status='active')",
    )
    .bind(network.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::storage(id, e))?;
    Ok(Json(FreezeStateResponse {
        network_id: network.id,
        latest_ledger: row.0,
        frozen_key_count: row.1,
        bypass_count: row.2,
        active_incident_count: row.3,
    }))
}

#[utoipa::path(get, path = "/api/v1/frozen-keys", params(ListQuery), responses((status = 200, body = FrozenKeysResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn frozen_keys(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ListQuery>,
) -> Result<Json<FrozenKeysResponse>, ApiError> {
    let (page, page_size) = q.pagination(id)?;
    let network = selected_network(&pool, id, q.network_id).await?;
    let rows: Vec<(Vec<u8>, i64, i64, Option<String>)> = sqlx::query_as(
        "SELECT key_hash, active_since, last_changed, evidence_ref FROM current_frozen_keys WHERE network_id=$1 ORDER BY key_hash LIMIT $2 OFFSET $3"
    ).bind(network.id).bind(page_size).bind((page-1)*page_size).fetch_all(&pool).await.map_err(|e| ApiError::storage(id, e))?;
    let items = rows
        .into_iter()
        .map(|r| FrozenKeyResponse {
            id: hex::encode(r.0),
            network_id: network.id,
            active_since: r.1,
            last_changed: r.2,
            evidence_ref: r.3,
        })
        .collect();
    Ok(Json(FrozenKeysResponse {
        items,
        page,
        page_size,
    }))
}

#[utoipa::path(get, path = "/api/v1/frozen-keys/{id}", params(("id" = String, Path, description = "64-character hexadecimal key hash"), ScopeQuery), responses((status = 200, body = FrozenKeyResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn frozen_key(
    State(pool): State<PgPool>,
    Extension(request_id): Extension<RequestId>,
    Path(key_id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<FrozenKeyResponse>, ApiError> {
    let bytes = hex::decode(&key_id).map_err(|_| {
        ApiError::bad_request(request_id, "id must be a 64-character hexadecimal hash")
    })?;
    if bytes.len() != 32 {
        return Err(ApiError::bad_request(
            request_id,
            "id must be a 64-character hexadecimal hash",
        ));
    }
    let network = selected_network(&pool, request_id, q.network_id).await?;
    let row: Option<(i64, i64, Option<String>)> = sqlx::query_as("SELECT active_since, last_changed, evidence_ref FROM current_frozen_keys WHERE network_id=$1 AND key_hash=$2")
        .bind(network.id).bind(&bytes).fetch_optional(&pool).await.map_err(|e| ApiError::storage(request_id, e))?;
    let (active_since, last_changed, evidence_ref) =
        row.ok_or_else(|| ApiError::not_found(request_id, "Frozen key not found"))?;
    Ok(Json(FrozenKeyResponse {
        id: hex::encode(bytes),
        network_id: network.id,
        active_since,
        last_changed,
        evidence_ref,
    }))
}

#[utoipa::path(get, path = "/api/v1/bypasses", params(ListQuery), responses((status = 200, body = BypassesResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn bypasses(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ListQuery>,
) -> Result<Json<BypassesResponse>, ApiError> {
    let (page, page_size) = q.pagination(id)?;
    let network = selected_network(&pool, id, q.network_id).await?;
    let rows: Vec<(String, i64, i64, Option<String>)> = sqlx::query_as("SELECT tx_hash, active_since, last_changed, evidence_ref FROM current_bypasses WHERE network_id=$1 ORDER BY tx_hash LIMIT $2 OFFSET $3")
        .bind(network.id).bind(page_size).bind((page-1)*page_size).fetch_all(&pool).await.map_err(|e| ApiError::storage(id, e))?;
    let items = rows
        .into_iter()
        .map(|r| BypassResponse {
            tx_hash: r.0,
            network_id: network.id,
            active_since: r.1,
            last_changed: r.2,
            evidence_ref: r.3,
        })
        .collect();
    Ok(Json(BypassesResponse {
        items,
        page,
        page_size,
    }))
}

#[derive(FromRow)]
struct IncidentRow {
    id: Uuid,
    network_id: Uuid,
    basis: String,
    opened_ledger: i64,
    opened_close_time: Option<chrono::DateTime<chrono::Utc>>,
    closed_ledger: Option<i64>,
    closed_close_time: Option<chrono::DateTime<chrono::Utc>>,
    status: String,
}

impl From<IncidentRow> for IncidentResponse {
    fn from(r: IncidentRow) -> Self {
        Self {
            id: r.id,
            network_id: r.network_id,
            basis: r.basis,
            opened_ledger: r.opened_ledger,
            opened_close_time: r.opened_close_time.map(|v| v.to_rfc3339()),
            closed_ledger: r.closed_ledger,
            closed_close_time: r.closed_close_time.map(|v| v.to_rfc3339()),
            status: r.status,
        }
    }
}

const INCIDENT_COLUMNS: &str = "id, network_id, basis, opened_ledger, opened_close_time, closed_ledger, closed_close_time, status";

#[utoipa::path(get, path = "/api/v1/incidents", params(ListQuery), responses((status = 200, body = IncidentsResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn incidents(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ListQuery>,
) -> Result<Json<IncidentsResponse>, ApiError> {
    let (page, page_size) = q.pagination(id)?;
    let network = selected_network(&pool, id, q.network_id).await?;
    let sql = format!(
        "SELECT {INCIDENT_COLUMNS} FROM incidents WHERE network_id=$1 ORDER BY opened_ledger DESC, id LIMIT $2 OFFSET $3"
    );
    let rows = sqlx::query_as::<_, IncidentRow>(&sql)
        .bind(network.id)
        .bind(page_size)
        .bind((page - 1) * page_size)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::storage(id, e))?;
    Ok(Json(IncidentsResponse {
        items: rows.into_iter().map(Into::into).collect(),
        page,
        page_size,
    }))
}

fn parse_incident_id(value: &str, request_id: RequestId) -> Result<Uuid, ApiError> {
    Uuid::parse_str(value).map_err(|_| ApiError::bad_request(request_id, "id must be a UUID"))
}

async fn find_incident(
    pool: &PgPool,
    request_id: RequestId,
    incident_id: Uuid,
    network_id: Uuid,
) -> Result<IncidentResponse, ApiError> {
    let sql = format!("SELECT {INCIDENT_COLUMNS} FROM incidents WHERE id=$1 AND network_id=$2");
    sqlx::query_as::<_, IncidentRow>(&sql)
        .bind(incident_id)
        .bind(network_id)
        .fetch_optional(pool)
        .await
        .map_err(|e| ApiError::storage(request_id, e))?
        .map(Into::into)
        .ok_or_else(|| ApiError::not_found(request_id, "Incident not found"))
}

#[utoipa::path(get, path = "/api/v1/incidents/{id}", params(("id" = Uuid, Path), ScopeQuery), responses((status = 200, body = IncidentResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn incident(
    State(pool): State<PgPool>,
    Extension(request_id): Extension<RequestId>,
    Path(id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<IncidentResponse>, ApiError> {
    let incident_id = parse_incident_id(&id, request_id)?;
    let network = selected_network(&pool, request_id, q.network_id).await?;
    Ok(Json(
        find_incident(&pool, request_id, incident_id, network.id).await?,
    ))
}

#[derive(FromRow)]
struct TimelineRow {
    id: i32,
    incident_id: Uuid,
    ledger_sequence: i64,
    kind: String,
    evidence_ref: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(get, path = "/api/v1/incidents/{id}/timeline", params(("id" = Uuid, Path), ListQuery), responses((status = 200, body = TimelineResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn timeline(
    State(pool): State<PgPool>,
    Extension(request_id): Extension<RequestId>,
    Path(id): Path<String>,
    Query(q): Query<ListQuery>,
) -> Result<Json<TimelineResponse>, ApiError> {
    let incident_id = parse_incident_id(&id, request_id)?;
    let (page, page_size) = q.pagination(request_id)?;
    let network = selected_network(&pool, request_id, q.network_id).await?;
    find_incident(&pool, request_id, incident_id, network.id).await?;
    let rows = sqlx::query_as::<_, TimelineRow>("SELECT id, incident_id, ledger_sequence, kind, evidence_ref, created_at FROM incident_events WHERE incident_id=$1 ORDER BY ledger_sequence, id LIMIT $2 OFFSET $3")
        .bind(incident_id).bind(page_size).bind((page-1)*page_size).fetch_all(&pool).await.map_err(|e| ApiError::storage(request_id, e))?;
    let items = rows
        .into_iter()
        .map(|r| TimelineEventResponse {
            id: r.id,
            incident_id: r.incident_id,
            ledger_sequence: r.ledger_sequence,
            kind: r.kind,
            evidence_ref: r.evidence_ref,
            created_at: r.created_at.to_rfc3339(),
        })
        .collect();
    Ok(Json(TimelineResponse {
        items,
        page,
        page_size,
    }))
}

#[utoipa::path(get, path = "/api/v1/status", params(ScopeQuery), responses((status = 200, body = StatusResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn status(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<StatusResponse>, ApiError> {
    let network = selected_network(&pool, id, q.network_id).await?;
    let row: Option<(String, i64, chrono::DateTime<chrono::Utc>)> = sqlx::query_as("SELECT stream, last_complete_ledger, updated_at FROM ingestion_checkpoints WHERE network_id=$1 ORDER BY updated_at DESC, stream LIMIT 1")
        .bind(network.id).fetch_optional(&pool).await.map_err(|e| ApiError::storage(id, e))?;
    let (stream, last_complete_ledger, updated_at) =
        row.ok_or_else(|| ApiError::not_found(id, "No indexer checkpoint found"))?;
    Ok(Json(StatusResponse {
        network_id: network.id,
        stream,
        last_complete_ledger,
        updated_at: updated_at.to_rfc3339(),
    }))
}
