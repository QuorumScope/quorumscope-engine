use thiserror::Error;

#[derive(Debug, Error)]
pub enum XdrError {
    #[error("base64 decode error: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("XDR decode error: {0}")]
    Decode(#[from] stellar_xdr::Error),
    #[error("malformed input")]
    MalformedInput,
    #[error("unsupported ledger key type")]
    UnsupportedLedgerKey,
}
