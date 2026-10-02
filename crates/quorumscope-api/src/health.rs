use axum::response::IntoResponse;
use axum::{Json, Router, routing::get};
use serde_json::json;

async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz))
}
