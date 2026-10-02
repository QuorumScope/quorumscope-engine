pub mod health;
pub mod error;
pub mod middleware;
pub mod incidents;
pub mod metrics;

use axum::Router;
use sqlx::PgPool;
use incidents::AppState;

pub fn app(pool: PgPool) -> Router {
    let state = AppState { pool };
    Router::new().layer(axum::middleware::from_fn(middleware::track_metrics)).merge(health::router().merge(metrics::router()).merge(incidents::router(state)))
}
