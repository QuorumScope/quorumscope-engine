use axum::{Extension, Json, extract::State};
use serde::Serialize;
use sqlx::PgPool;
use utoipa::ToSchema;

use crate::error::{ApiError, RequestId};

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[utoipa::path(get, path = "/health/live", responses((status = 200, body = HealthResponse)))]
pub async fn live() -> Json<HealthResponse> {
    Json(HealthResponse { status: "live" })
}

#[utoipa::path(get, path = "/health/ready", responses((status = 200, body = HealthResponse), (status = 503, body = crate::error::ErrorEnvelope)))]
pub async fn ready(
    State(pool): State<PgPool>,
    Extension(id): Extension<RequestId>,
) -> Result<Json<HealthResponse>, ApiError> {
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| ApiError::unavailable(id, e))?;
    Ok(Json(HealthResponse { status: "ready" }))
}
