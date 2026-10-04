use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Copy)]
pub struct RequestId(pub Uuid);

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorEnvelope {
    pub error: ErrorBody,
    pub request_id: Uuid,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
}

pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
    pub request_id: Uuid,
}

impl ApiError {
    pub fn bad_request(id: RequestId, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_input",
            message: message.into(),
            request_id: id.0,
        }
    }

    pub fn not_found(id: RequestId, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
            request_id: id.0,
        }
    }

    pub fn storage(id: RequestId, error: sqlx::Error) -> Self {
        tracing::error!(request_id = %id.0, error = %error, "API storage read failed");
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "storage_error",
            message: "Storage read failed".into(),
            request_id: id.0,
        }
    }

    pub fn unavailable(id: RequestId, error: sqlx::Error) -> Self {
        tracing::error!(request_id = %id.0, error = %error, "API readiness check failed");
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "storage_unavailable",
            message: "Storage unavailable".into(),
            request_id: id.0,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                },
                request_id: self.request_id,
            }),
        )
            .into_response()
    }
}
