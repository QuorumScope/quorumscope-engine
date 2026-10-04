use axum::{
    Extension, Json,
    extract::{Path, Query, State},
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use base64::Engine as _;
use std::sync::Arc;

use crate::error::{ApiError, RequestId};
use crate::freshness::{self, ApiConfig, StateFreshness};

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

#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct KeyListQuery {
    /// Network UUID. When omitted, the first network by name is selected.
    pub network_id: Option<Uuid>,
    /// One-based page number. Defaults to 1.
    pub page: Option<i64>,
    /// Items per page, from 1 to 100. Defaults to 50.
    pub page_size: Option<i64>,
    /// Filter by `account`, `trustline`, `contract_data` or `contract_code`.
    pub kind: Option<String>,
    /// Defaults to true. Set to false to include keys that are no longer frozen.
    pub active: Option<bool>,
}

#[derive(Clone, FromRow)]
pub struct NetworkRow {
    pub id: Uuid,
    pub name: String,
    pub passphrase: String,
}

#[derive(Serialize, ToSchema)]
pub struct NetworkResponse {
    pub id: Uuid,
    pub name: String,
    pub passphrase: String,
    pub freshness: StateFreshness,
}

async fn selected_network(
    pool: &PgPool,
    id: RequestId,
    network_id: Option<Uuid>,
) -> Result<NetworkRow, ApiError> {
    sqlx::query_as::<_, NetworkRow>("SELECT id, name, passphrase FROM networks WHERE ($1::uuid IS NULL OR id = $1) ORDER BY name LIMIT 1")
        .bind(network_id).fetch_optional(pool).await.map_err(|e| ApiError::storage(id, e))?
        .ok_or_else(|| ApiError::not_found(id, "Network not found"))
}

pub(crate) async fn network_scope(
    pool: &PgPool,
    id: RequestId,
    network_id: Option<Uuid>,
) -> Result<NetworkRow, ApiError> {
    selected_network(pool, id, network_id).await
}

async fn freshness_for(
    pool: &PgPool,
    config: &ApiConfig,
    id: RequestId,
    network_id: Uuid,
) -> Result<StateFreshness, ApiError> {
    freshness::load(pool, config, network_id)
        .await
        .map_err(|e| ApiError::storage(id, e))
}

#[derive(Serialize, ToSchema)]
pub struct FreezeStateResponse {
    pub freshness: StateFreshness,
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
    /// True when the key is in the current freeze set.
    pub active: bool,
    /// `account`, `trustline`, `contract_data` or `contract_code`. Absent when the
    /// indexer has not stored the key content.
    pub kind: Option<String>,
    /// Decoded key fields as stored by the indexer.
    #[schema(value_type = Option<Object>)]
    pub decoded: Option<serde_json::Value>,
    /// Canonical ledger key XDR, base64 encoded.
    pub canonical_xdr: Option<String>,
    /// Ledger of the freeze that is currently in effect. Null when not active.
    pub active_since: Option<i64>,
    /// Ledger of the first recorded freeze of this key.
    pub first_frozen_ledger: Option<i64>,
    /// Ledger of the latest recorded change.
    pub last_changed: Option<i64>,
    pub evidence_ref: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct KeyChangeResponse {
    pub ledger_sequence: i64,
    /// `freeze` or `unfreeze`.
    pub action: String,
    pub result: String,
    /// Configuration snapshot that supported the change, when recorded.
    pub evidence_ref: Option<String>,
    pub recorded_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct FrozenKeyDetailResponse {
    #[serde(flatten)]
    pub key: FrozenKeyResponse,
    /// Every recorded freeze and unfreeze, oldest first. Covers only ledgers the
    /// indexer has observed.
    pub history: Vec<KeyChangeResponse>,
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
    pub freshness: StateFreshness,
    pub network_id: Uuid,
    pub stream: String,
    pub last_complete_ledger: i64,
    pub updated_at: String,
}

#[utoipa::path(get, path = "/api/v1/network", params(ScopeQuery), responses((status = 200, body = NetworkResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn network(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<ApiConfig>>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<NetworkResponse>, ApiError> {
    let network = selected_network(&pool, id, q.network_id).await?;
    let freshness = freshness_for(&pool, &config, id, network.id).await?;
    Ok(Json(NetworkResponse {
        id: network.id,
        name: network.name,
        passphrase: network.passphrase,
        freshness,
    }))
}

#[utoipa::path(get, path = "/api/v1/freeze-state", params(ScopeQuery), responses((status = 200, body = FreezeStateResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn freeze_state(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<ApiConfig>>,
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
    let freshness = freshness_for(&pool, &config, id, network.id).await?;
    Ok(Json(FreezeStateResponse {
        freshness,
        network_id: network.id,
        latest_ledger: row.0,
        frozen_key_count: row.1,
        bypass_count: row.2,
        active_incident_count: row.3,
    }))
}

type HistoryRow = (
    i64,
    String,
    String,
    Option<String>,
    chrono::DateTime<chrono::Utc>,
);

const KEY_KINDS: [&str; 4] = ["account", "trustline", "contract_data", "contract_code"];

type KeyRow = (
    Vec<u8>,
    Option<String>,
    Option<serde_json::Value>,
    Option<Vec<u8>>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    bool,
);

const KEY_SELECT: &str = "SELECT ids.key_hash, k.key_kind, k.decoded_json, k.canonical_key_xdr, c.active_since, \
    (SELECT MIN(f.ledger_sequence) FROM freeze_changes f WHERE f.network_id=$1 AND f.key_hash=ids.key_hash AND f.action='freeze'), \
    COALESCE(c.last_changed, (SELECT MAX(f.ledger_sequence) FROM freeze_changes f WHERE f.network_id=$1 AND f.key_hash=ids.key_hash)), \
    c.evidence_ref, (c.key_hash IS NOT NULL) \
    FROM ids \
    LEFT JOIN ledger_keys k ON k.network_id=$1 AND k.key_hash=ids.key_hash \
    LEFT JOIN current_frozen_keys c ON c.network_id=$1 AND c.key_hash=ids.key_hash";

fn key_response(network_id: Uuid, r: KeyRow) -> FrozenKeyResponse {
    FrozenKeyResponse {
        id: hex::encode(r.0),
        network_id,
        active: r.8,
        kind: r.1,
        decoded: r.2,
        canonical_xdr: r
            .3
            .map(|x| base64::engine::general_purpose::STANDARD.encode(x)),
        active_since: r.4,
        first_frozen_ledger: r.5,
        last_changed: r.6,
        evidence_ref: r.7,
    }
}

#[utoipa::path(get, path = "/api/v1/frozen-keys", params(KeyListQuery), responses((status = 200, body = FrozenKeysResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn frozen_keys(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<KeyListQuery>,
) -> Result<Json<FrozenKeysResponse>, ApiError> {
    let (page, page_size) = ListQuery {
        network_id: q.network_id,
        page: q.page,
        page_size: q.page_size,
    }
    .pagination(id)?;
    if let Some(kind) = &q.kind
        && !KEY_KINDS.contains(&kind.as_str())
    {
        return Err(ApiError::bad_request(
            id,
            "kind must be account, trustline, contract_data or contract_code",
        ));
    }
    let network = selected_network(&pool, id, q.network_id).await?;
    let sql = format!(
        "WITH ids AS (SELECT key_hash FROM current_frozen_keys WHERE network_id=$1 \
           UNION SELECT key_hash FROM ledger_keys WHERE network_id=$1 AND NOT $3) \
         {KEY_SELECT} WHERE ($2::text IS NULL OR k.key_kind=$2) \
         ORDER BY ids.key_hash LIMIT $4 OFFSET $5"
    );
    let rows: Vec<KeyRow> = sqlx::query_as(&sql)
        .bind(network.id)
        .bind(&q.kind)
        .bind(q.active.unwrap_or(true))
        .bind(page_size)
        .bind((page - 1) * page_size)
        .fetch_all(&pool)
        .await
        .map_err(|e| ApiError::storage(id, e))?;
    let items = rows
        .into_iter()
        .map(|r| key_response(network.id, r))
        .collect();
    Ok(Json(FrozenKeysResponse {
        items,
        page,
        page_size,
    }))
}

#[utoipa::path(get, path = "/api/v1/frozen-keys/{id}", params(("id" = String, Path, description = "64-character hexadecimal key hash"), ScopeQuery), responses((status = 200, body = FrozenKeyDetailResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn frozen_key(
    State(pool): State<PgPool>,
    Extension(request_id): Extension<RequestId>,
    Path(key_id): Path<String>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<FrozenKeyDetailResponse>, ApiError> {
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
    let sql = format!("WITH ids AS (SELECT $2::bytea AS key_hash) {KEY_SELECT}");
    let row: KeyRow = sqlx::query_as(&sql)
        .bind(network.id)
        .bind(&bytes)
        .fetch_one(&pool)
        .await
        .map_err(|e| ApiError::storage(request_id, e))?;
    let history: Vec<HistoryRow> = sqlx::query_as(
        "SELECT ledger_sequence, action, result, evidence_ref, created_at FROM freeze_changes \
             WHERE network_id=$1 AND key_hash=$2 ORDER BY ledger_sequence, created_at",
    )
    .bind(network.id)
    .bind(&bytes)
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::storage(request_id, e))?;
    if !row.8 && row.1.is_none() && history.is_empty() {
        return Err(ApiError::not_found(request_id, "Frozen key not found"));
    }
    Ok(Json(FrozenKeyDetailResponse {
        key: key_response(network.id, row),
        history: history
            .into_iter()
            .map(|h| KeyChangeResponse {
                ledger_sequence: h.0,
                action: h.1,
                result: h.2,
                evidence_ref: h.3,
                recorded_at: h.4.to_rfc3339(),
            })
            .collect(),
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
    Extension(config): Extension<Arc<ApiConfig>>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ScopeQuery>,
) -> Result<Json<StatusResponse>, ApiError> {
    let network = selected_network(&pool, id, q.network_id).await?;
    let row: Option<(String, i64, chrono::DateTime<chrono::Utc>)> = sqlx::query_as("SELECT stream, last_complete_ledger, updated_at FROM ingestion_checkpoints WHERE network_id=$1 ORDER BY updated_at DESC, stream LIMIT 1")
        .bind(network.id).fetch_optional(&pool).await.map_err(|e| ApiError::storage(id, e))?;
    let (stream, last_complete_ledger, updated_at) =
        row.ok_or_else(|| ApiError::not_found(id, "No indexer checkpoint found"))?;
    let freshness = freshness_for(&pool, &config, id, network.id).await?;
    Ok(Json(StatusResponse {
        freshness,
        network_id: network.id,
        stream,
        last_complete_ledger,
        updated_at: updated_at.to_rfc3339(),
    }))
}
