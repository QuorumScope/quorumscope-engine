pub mod error;
pub mod health;
pub mod metrics;
pub mod middleware;
pub mod read;

use axum::{Extension, Router, routing::get};
use sqlx::PgPool;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "QuorumScope API", version = "1.0.0"),
    paths(
        health::live,
        health::ready,
        read::network,
        read::freeze_state,
        read::frozen_keys,
        read::frozen_key,
        read::bypasses,
        read::incidents,
        read::incident,
        read::timeline,
        read::status
    ),
    components(schemas(
        error::ErrorEnvelope,
        error::ErrorBody,
        health::HealthResponse,
        read::NetworkResponse,
        read::FreezeStateResponse,
        read::FrozenKeyResponse,
        read::FrozenKeysResponse,
        read::BypassResponse,
        read::BypassesResponse,
        read::IncidentResponse,
        read::IncidentsResponse,
        read::TimelineEventResponse,
        read::TimelineResponse,
        read::StatusResponse
    ))
)]
pub struct ApiDoc;

pub fn app(pool: PgPool) -> Router {
    Router::new()
        .route("/health/live", get(health::live))
        .route("/health/ready", get(health::ready))
        .route("/api/v1/network", get(read::network))
        .route("/api/v1/freeze-state", get(read::freeze_state))
        .route("/api/v1/frozen-keys", get(read::frozen_keys))
        .route("/api/v1/frozen-keys/{id}", get(read::frozen_key))
        .route("/api/v1/bypasses", get(read::bypasses))
        .route("/api/v1/incidents", get(read::incidents))
        .route("/api/v1/incidents/{id}", get(read::incident))
        .route("/api/v1/incidents/{id}/timeline", get(read::timeline))
        .route("/api/v1/status", get(read::status))
        .route("/openapi.json", get(openapi_json))
        .route("/metrics", get(metrics::metrics_handler))
        .fallback(not_found)
        .with_state(pool)
        .layer(axum::middleware::from_fn(middleware::request_id))
        .layer(axum::middleware::from_fn(middleware::track_metrics))
}

async fn openapi_json() -> axum::Json<utoipa::openapi::OpenApi> {
    axum::Json(ApiDoc::openapi())
}

async fn not_found(Extension(id): Extension<error::RequestId>) -> error::ApiError {
    error::ApiError::not_found(id, "Route not found")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn committed_openapi_matches_generated_contract() {
        let committed: serde_json::Value =
            serde_json::from_str(include_str!("../../../openapi/openapi.json")).unwrap();
        let generated = serde_json::to_value(ApiDoc::openapi()).unwrap();
        assert_eq!(committed, generated);
    }
}
