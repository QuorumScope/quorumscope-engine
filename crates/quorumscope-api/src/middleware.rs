use axum::{
    Json,
    extract::Request,
    http::{HeaderValue, header::HeaderName},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::time::Instant;
use uuid::Uuid;

use crate::error::{ErrorBody, ErrorEnvelope, RequestId};

pub async fn request_id(mut req: Request, next: Next) -> Response {
    let id = Uuid::new_v4();
    req.extensions_mut().insert(RequestId(id));
    let mut response = next.run(req).await;
    if response.status().is_client_error() || response.status().is_server_error() {
        let is_json = response
            .headers()
            .get(axum::http::header::CONTENT_TYPE)
            .is_some_and(|value| value.as_bytes().starts_with(b"application/json"));
        if !is_json {
            let status = response.status();
            let (code, message) = match status {
                axum::http::StatusCode::METHOD_NOT_ALLOWED => {
                    ("method_not_allowed", "Method not allowed")
                }
                axum::http::StatusCode::NOT_FOUND => ("not_found", "Route not found"),
                _ if status.is_client_error() => ("invalid_input", "Invalid request"),
                _ => ("internal_error", "Internal server error"),
            };
            response = (
                status,
                Json(ErrorEnvelope {
                    error: ErrorBody {
                        code,
                        message: message.into(),
                    },
                    request_id: id,
                }),
            )
                .into_response();
        }
    }
    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&id.to_string()).expect("UUID is a valid header value"),
    );
    response
}

pub async fn track_metrics(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let path = req.uri().path().to_owned();
    let start = Instant::now();
    let response = next.run(req).await;
    tracing::info!(method = %method, path = %path, status = response.status().as_u16(), latency_ms = start.elapsed().as_millis(), "Request processed");
    response
}
