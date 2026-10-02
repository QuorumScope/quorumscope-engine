use axum::{routing::get, Router};
use axum::response::IntoResponse;

async fn metrics_handler() -> impl IntoResponse {
    // Simple prometheus-style text output
    let mut output = String::new();
    output.push_str("# HELP quorumscope_up Whether the service is up\n");
    output.push_str("# TYPE quorumscope_up gauge\n");
    output.push_str("quorumscope_up 1\n");
    
    output
}

pub fn router() -> Router {
    Router::new().route("/metrics", get(metrics_handler))
}
