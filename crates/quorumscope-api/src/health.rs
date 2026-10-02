use axum::{routing::get, Router, Json};
use axum::response::IntoResponse;
use serde_json::json;

async fn healthz() -> impl IntoResponse {
    Json(json!({ "status": "ok" }))
}

pub fn router() -> Router {
    Router::new().route("/healthz", get(healthz))
}
