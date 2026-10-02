use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("Failed to generate report: {0}")]
    GenerationError(String),
}
