use axum::{Extension, Json, extract::State};
use quorumscope_domain::bypass::BypassHash;
use quorumscope_domain::freeze::FrozenKeyId;
use quorumscope_domain::preflight::{
    PreflightConfidence, PreflightFinding, PreflightResult, PreflightStatus,
};
use quorumscope_xdr::transaction::DecodedTransactionEnvelope;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashSet;
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::error::{ApiError, RequestId};
use crate::freshness::{self, ApiConfig, FreshnessStatus, StateFreshness};
use crate::read::network_scope;

/// Largest accepted request body in bytes. Stellar transaction envelopes are far smaller.
pub const MAX_BODY_BYTES: usize = 256 * 1024;

#[derive(Deserialize, ToSchema)]
pub struct PreflightRequest {
    /// Base64 transaction envelope XDR. Whitespace and line breaks are ignored.
    pub transaction_xdr: String,
    /// Network UUID. When omitted, the first network by name is selected.
    pub network_id: Option<Uuid>,
}

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreflightStatusResponse {
    Clear,
    BlockedValidation,
    AllowedByBypass,
    ApplyTimeRisk,
    DexConditional,
    InvalidInput,
    UnsupportedAnalysis,
    StateUnavailable,
}

#[derive(Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PreflightConfidenceResponse {
    Deterministic,
    Conditional,
    InsufficientInformation,
}

impl From<PreflightStatus> for PreflightStatusResponse {
    fn from(s: PreflightStatus) -> Self {
        match s {
            PreflightStatus::Clear => Self::Clear,
            PreflightStatus::BlockedValidation => Self::BlockedValidation,
            PreflightStatus::AllowedByBypass => Self::AllowedByBypass,
            PreflightStatus::ApplyTimeRisk => Self::ApplyTimeRisk,
            PreflightStatus::DexConditional => Self::DexConditional,
            PreflightStatus::InvalidInput => Self::InvalidInput,
            PreflightStatus::UnsupportedAnalysis => Self::UnsupportedAnalysis,
            PreflightStatus::StateUnavailable => Self::StateUnavailable,
        }
    }
}

impl From<PreflightConfidence> for PreflightConfidenceResponse {
    fn from(c: PreflightConfidence) -> Self {
        match c {
            PreflightConfidence::Deterministic => Self::Deterministic,
            PreflightConfidence::Conditional => Self::Conditional,
            PreflightConfidence::InsufficientInformation => Self::InsufficientInformation,
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct FindingResponse {
    pub status: PreflightStatusResponse,
    pub confidence: PreflightConfidenceResponse,
    /// Frozen key hashes (hex) this finding implicates.
    pub implicated_keys: Vec<String>,
    /// Where in the transaction the finding applies.
    pub protocol_path: String,
    pub explanation: String,
}

impl From<&PreflightFinding> for FindingResponse {
    fn from(f: &PreflightFinding) -> Self {
        Self {
            status: f.status.into(),
            confidence: f.confidence.into(),
            implicated_keys: f.implicated_keys.iter().map(|k| k.to_string()).collect(),
            protocol_path: f.protocol_path.clone(),
            explanation: f.explanation.clone(),
        }
    }
}

#[derive(Serialize, ToSchema)]
pub struct PreflightResponse {
    pub request_id: Uuid,
    pub network_id: Uuid,
    pub status: PreflightStatusResponse,
    pub confidence: PreflightConfidenceResponse,
    /// Hex transaction content hash for this network. Absent when the input was invalid.
    pub transaction_hash: Option<String>,
    /// True when the transaction content hash is in the active bypass set.
    pub is_bypassed: bool,
    /// Ledger the freeze state was read at. Absent when state is unavailable.
    pub source_ledger: Option<i64>,
    pub freshness: StateFreshness,
    pub findings: Vec<FindingResponse>,
}

fn finding(
    status: PreflightStatus,
    confidence: PreflightConfidence,
    path: &str,
    explanation: String,
) -> PreflightFinding {
    PreflightFinding {
        status,
        confidence,
        implicated_keys: vec![],
        protocol_path: path.to_string(),
        explanation,
    }
}

fn decode(input: &str) -> Result<DecodedTransactionEnvelope, String> {
    let compact: String = input.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() {
        return Err("transaction_xdr is empty.".into());
    }
    let bytes = quorumscope_xdr::codec::decode_base64(&compact)
        .map_err(|_| "transaction_xdr is not valid base64.".to_string())?;
    quorumscope_xdr::transaction::decode_transaction_envelope(&bytes)
        .map_err(|_| "transaction_xdr is not a valid transaction envelope.".to_string())
}

#[utoipa::path(
    post,
    path = "/api/v1/preflight",
    request_body = PreflightRequest,
    responses(
        (status = 200, description = "Analysis result. Check `status`; malformed XDR returns `invalid_input` here.", body = PreflightResponse),
        (status = 400, body = crate::error::ErrorEnvelope),
        (status = 404, body = crate::error::ErrorEnvelope),
        (status = 413, body = crate::error::ErrorEnvelope),
        (status = 500, body = crate::error::ErrorEnvelope)
    )
)]
pub async fn preflight(
    State(pool): State<PgPool>,
    Extension(config): Extension<Arc<ApiConfig>>,
    Extension(request_id): Extension<RequestId>,
    Json(req): Json<PreflightRequest>,
) -> Result<Json<PreflightResponse>, ApiError> {
    let network = network_scope(&pool, request_id, req.network_id).await?;
    let state = freshness::load(&pool, &config, network.id)
        .await
        .map_err(|e| ApiError::storage(request_id, e))?;

    let respond = |status: PreflightStatus,
                   confidence: PreflightConfidence,
                   hash: Option<String>,
                   bypassed: bool,
                   findings: Vec<PreflightFinding>| {
        Json(PreflightResponse {
            request_id: request_id.0,
            network_id: network.id,
            status: status.into(),
            confidence: confidence.into(),
            transaction_hash: hash,
            is_bypassed: bypassed,
            source_ledger: if status == PreflightStatus::StateUnavailable {
                None
            } else {
                state.source_ledger
            },
            freshness: state.clone(),
            findings: findings.iter().map(Into::into).collect(),
        })
    };

    let decoded = match decode(&req.transaction_xdr) {
        Ok(d) => d,
        Err(reason) => {
            return Ok(respond(
                PreflightStatus::InvalidInput,
                PreflightConfidence::Deterministic,
                None,
                false,
                vec![finding(
                    PreflightStatus::InvalidInput,
                    PreflightConfidence::Deterministic,
                    "transaction_xdr",
                    reason,
                )],
            ));
        }
    };

    let unavailable = match state.status {
        FreshnessStatus::Unknown => {
            Some("QuorumScope has no indexed freeze state for this network.")
        }
        FreshnessStatus::Stale => Some(
            "QuorumScope has not observed this network recently, so its freeze state may be out of date.",
        ),
        FreshnessStatus::Current | FreshnessStatus::IndexingBehind => None,
    };
    if let Some(reason) = unavailable {
        return Ok(respond(
            PreflightStatus::StateUnavailable,
            PreflightConfidence::InsufficientInformation,
            None,
            false,
            vec![finding(
                PreflightStatus::StateUnavailable,
                PreflightConfidence::InsufficientInformation,
                "freeze state",
                format!("{reason} No clear result was produced."),
            )],
        ));
    }

    let frozen: HashSet<FrozenKeyId> = sqlx::query_scalar::<_, Vec<u8>>(
        "SELECT key_hash FROM current_frozen_keys WHERE network_id=$1",
    )
    .bind(network.id)
    .fetch_all(&pool)
    .await
    .map_err(|e| ApiError::storage(request_id, e))?
    .into_iter()
    .filter_map(|b| <[u8; 32]>::try_from(b).ok().map(FrozenKeyId::new))
    .collect();
    let bypasses: HashSet<BypassHash> =
        sqlx::query_scalar::<_, String>("SELECT tx_hash FROM current_bypasses WHERE network_id=$1")
            .bind(network.id)
            .fetch_all(&pool)
            .await
            .map_err(|e| ApiError::storage(request_id, e))?
            .into_iter()
            .filter_map(|h| {
                hex::decode(h)
                    .ok()
                    .and_then(|b| BypassHash::from_slice(&b).ok())
            })
            .collect();

    let analysis =
        quorumscope_preflight::analyze(&decoded.envelope, &network.passphrase, &frozen, &bypasses);
    let status = PreflightResult::derive_status(&analysis.findings, analysis.bypassed);
    let confidence = PreflightResult::derive_confidence(&analysis.findings);
    Ok(respond(
        status,
        confidence,
        Some(analysis.transaction_hash),
        analysis.bypassed,
        analysis.findings,
    ))
}
