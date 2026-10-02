use crate::ledger::LedgerError;
use crate::network::NetworkError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error(transparent)]
    Network(#[from] NetworkError),
    #[error(transparent)]
    Ledger(#[from] LedgerError),
}
