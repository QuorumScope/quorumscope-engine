use axum::{routing::get, Router, extract::{Path, State}, Json};
use axum::response::IntoResponse;
use uuid::Uuid;
use serde_json::json;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

async fn list_incidents(State(state): State<AppState>) -> impl IntoResponse {
    // Basic extraction
    Json(json!([]))
}

async fn get_incident(State(state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "id": id }))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/incidents", get(list_incidents))
        .route("/incidents/:id", get(get_incident))
        .with_state(state)
}
