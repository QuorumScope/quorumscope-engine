use thiserror::Error;

#[derive(Debug, Error)]
pub enum RpcError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),
    #[error("RPC configuration error: {0}")]
    Config(String),
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}
