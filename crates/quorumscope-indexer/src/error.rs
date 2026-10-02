use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Pipeline error: {0}")]
    Pipeline(String),
}
