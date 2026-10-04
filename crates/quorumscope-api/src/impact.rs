use axum::{
    Extension, Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::error::{ApiError, RequestId};
use crate::read::network_scope;

/// Evidence classes the indexer currently stores.
const COLLECTED: [&str; 2] = ["direct", "protocol_derived"];
/// Classes defined by the contract that need transaction history, which is not indexed yet.
const NOT_COLLECTED: [&str; 3] = ["recently_observed", "dependency_observed", "inferred"];
const KEY_KINDS: [&str; 4] = ["account", "trustline", "contract_data", "contract_code"];

#[derive(Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ImpactQuery {
    /// Network UUID. When omitted, the first network by name is selected.
    pub network_id: Option<Uuid>,
    /// One-based page number. Defaults to 1.
    pub page: Option<i64>,
    /// Items per page, from 1 to 100. Defaults to 50.
    pub page_size: Option<i64>,
    /// `direct`, `protocol_derived`, `recently_observed`, `dependency_observed` or `inferred`.
    pub evidence_class: Option<String>,
    /// Restrict direct records to one key kind. Excludes records that have no key.
    pub key_kind: Option<String>,
    /// Restrict direct records to one 64-character hexadecimal key hash.
    pub key_id: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ObservationWindowResponse {
    pub first_ledger: i64,
    pub last_ledger: i64,
    /// Where the evidence came from.
    pub provider: String,
    /// False when ledgers inside the range may not have been observed.
    pub is_complete_for_range: bool,
}

#[derive(Serialize, ToSchema)]
pub struct ImpactRecordResponse {
    /// `direct` or `protocol_derived`. Other classes are not collected yet.
    pub evidence_class: String,
    pub key_id: Option<String>,
    pub key_kind: Option<String>,
    pub description: String,
    pub observation_window: ObservationWindowResponse,
    /// Configuration snapshot that supports the record, when recorded.
    pub evidence_ref: Option<String>,
    /// Structured values behind `description`.
    #[schema(value_type = Object)]
    pub details: serde_json::Value,
}

#[derive(Serialize, ToSchema)]
pub struct ImpactResponse {
    pub items: Vec<ImpactRecordResponse>,
    pub page: i64,
    pub page_size: i64,
    /// Evidence classes this engine stores and can return.
    pub collected_evidence_classes: Vec<String>,
    /// Evidence classes in the contract that have no stored evidence, so they never appear in `items`.
    pub uncollected_evidence_classes: Vec<String>,
}

type Row = (
    String,
    Option<Vec<u8>>,
    Option<String>,
    i64,
    i64,
    Option<String>,
    serde_json::Value,
);

#[utoipa::path(get, path = "/api/v1/impact", params(ImpactQuery), responses((status = 200, body = ImpactResponse), (status = 400, body = crate::error::ErrorEnvelope), (status = 404, body = crate::error::ErrorEnvelope), (status = 500, body = crate::error::ErrorEnvelope)))]
pub async fn impact(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
    Query(q): Query<ImpactQuery>,
) -> Result<Json<ImpactResponse>, ApiError> {
    let page = q.page.unwrap_or(1);
    let size = q.page_size.unwrap_or(50);
    if page < 1 || !(1..=100).contains(&size) || (page - 1).checked_mul(size).is_none() {
        return Err(ApiError::bad_request(id, "invalid pagination"));
    }
    if let Some(class) = &q.evidence_class
        && !COLLECTED.contains(&class.as_str())
        && !NOT_COLLECTED.contains(&class.as_str())
    {
        return Err(ApiError::bad_request(
            id,
            "evidence_class must be direct, protocol_derived, recently_observed, dependency_observed or inferred",
        ));
    }
    if let Some(kind) = &q.key_kind
        && !KEY_KINDS.contains(&kind.as_str())
    {
        return Err(ApiError::bad_request(
            id,
            "key_kind must be account, trustline, contract_data or contract_code",
        ));
    }
    let key_id = match &q.key_id {
        None => None,
        Some(value) => match hex::decode(value) {
            Ok(bytes) if bytes.len() == 32 => Some(bytes),
            _ => {
                return Err(ApiError::bad_request(
                    id,
                    "key_id must be a 64-character hexadecimal hash",
                ));
            }
        },
    };
    let network = network_scope(&pool, id, q.network_id).await?;
    let latest: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(last_complete_ledger) FROM ingestion_checkpoints WHERE network_id=$1",
    )
    .bind(network.id)
    .fetch_one(&pool)
    .await
    .map_err(|e| ApiError::storage(id, e))?;

    let rows: Vec<Row> = sqlx::query_as(
        "SELECT * FROM ( \
           SELECT 'direct'::text AS evidence_class, c.key_hash, k.key_kind, c.active_since AS first_ledger, \
                  COALESCE($6::bigint, c.last_changed) AS last_ledger, c.evidence_ref, \
                  jsonb_build_object('active_since', c.active_since, 'last_changed', c.last_changed) AS details \
           FROM current_frozen_keys c LEFT JOIN ledger_keys k ON k.network_id=c.network_id AND k.key_hash=c.key_hash \
           WHERE c.network_id=$1 AND ($3::text IS NULL OR k.key_kind=$3) AND ($4::bytea IS NULL OR c.key_hash=$4) \
          UNION ALL \
           SELECT 'protocol_derived', NULL::bytea, NULL::text, s.ledger_sequence, s.ledger_sequence, NULL::text, \
                  jsonb_build_object('incident_id', s.incident_id, 'frozen_accounts', s.total_frozen_accounts, \
                    'frozen_trustlines', s.total_frozen_trustlines, 'bypassed_transactions', s.total_bypassed_txs) \
           FROM impact_snapshots s JOIN incidents i ON i.id=s.incident_id \
           WHERE i.network_id=$1 AND $3::text IS NULL AND $4::bytea IS NULL \
         ) records WHERE ($2::text IS NULL OR evidence_class=$2) \
         ORDER BY evidence_class, last_ledger DESC, key_hash LIMIT $5 OFFSET $7",
    )
    .bind(network.id)
    .bind(&q.evidence_class)
    .bind(&q.key_kind)
    .bind(&key_id)
    .bind(size)
    .bind(latest)
    .bind((page - 1) * size)
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::storage(id, e))?;

    let items = rows
        .into_iter()
        .map(|(class, key, kind, first, last, evidence_ref, details)| {
            let direct = class == "direct";
            let description = if direct {
                format!("This key is in the active freeze set. It has been frozen since ledger {first}.")
            } else {
                format!(
                    "Frozen key counts recorded at ledger {first}, derived from the stored freeze set."
                )
            };
            ImpactRecordResponse {
                key_id: key.map(hex::encode),
                key_kind: kind,
                description,
                observation_window: ObservationWindowResponse {
                    first_ledger: first,
                    last_ledger: last,
                    provider: "quorumscope_indexer_rpc_state_polling".into(),
                    is_complete_for_range: false,
                },
                evidence_ref,
                details,
                evidence_class: class,
            }
        })
        .collect();
    Ok(Json(ImpactResponse {
        items,
        page,
        page_size: size,
        collected_evidence_classes: COLLECTED.iter().map(|s| s.to_string()).collect(),
        uncollected_evidence_classes: NOT_COLLECTED.iter().map(|s| s.to_string()).collect(),
    }))
}
