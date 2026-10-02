use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Path, State},
    routing::get,
};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

async fn list_incidents(State(_state): State<AppState>) -> impl IntoResponse {
    // Basic extraction
    Json(json!([]))
}

async fn get_incident(State(_state): State<AppState>, Path(id): Path<Uuid>) -> impl IntoResponse {
    Json(json!({ "id": id }))
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/incidents", get(list_incidents))
        .route("/incidents/:id", get(get_incident))
        .with_state(state)
}
