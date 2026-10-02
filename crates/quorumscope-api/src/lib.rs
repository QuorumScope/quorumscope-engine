pub mod error;
pub mod health;
pub mod incidents;
pub mod metrics;
pub mod middleware;

use axum::Router;
use incidents::AppState;
use sqlx::PgPool;

pub fn app(pool: PgPool) -> Router {
    let state = AppState { pool };
    Router::new()
        .layer(axum::middleware::from_fn(middleware::track_metrics))
        .merge(
            health::router()
                .merge(metrics::router())
                .merge(incidents::router(state)),
        )
}
