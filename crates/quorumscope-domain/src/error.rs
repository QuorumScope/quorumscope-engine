use thiserror::Error;
use crate::network::NetworkError;
use crate::ledger::LedgerError;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}
