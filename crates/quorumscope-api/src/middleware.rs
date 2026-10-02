use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

pub async fn track_metrics(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();

    let start = Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();

    tracing::info!(
        method = %method,
        path = %path,
        status = response.status().as_u16(),
        latency_ms = duration.as_millis(),
        "Request processed"
    );

    response
}
