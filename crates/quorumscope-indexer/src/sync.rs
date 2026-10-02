use crate::error::IndexerError;
use base64::Engine;
use quorumscope_rpc::client::RpcClient;
use quorumscope_rpc::get_ledger_entries::{GetLedgerEntriesRequest, GetLedgerEntriesResponse};
use quorumscope_xdr::config::frozen_ledger_keys_key;
use std::time::Duration;
use stellar_xdr::WriteXdr;

pub struct SyncTask {
    client: RpcClient,
    _interval: Duration,
}

impl SyncTask {
    pub fn new(client: RpcClient, interval: Duration) -> Self {
        Self {
            client,
            _interval: interval,
        }
    }

    pub async fn poll_configs(&self) -> Result<(), IndexerError> {
        let key = frozen_ledger_keys_key();
        let xdr_bytes = key
            .to_xdr(quorumscope_xdr::codec::default_limits())
            .map_err(|_| IndexerError::Pipeline("encode error".into()))?;

        let b64_key = base64::engine::general_purpose::STANDARD.encode(xdr_bytes);

        let req = GetLedgerEntriesRequest::new(1, vec![b64_key]);

        // Example logic:
        let res: GetLedgerEntriesResponse = self
            .client
            .send_request(&req)
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;

        tracing::info!("Received ledger entries: {:?}", res.result);
        Ok(())
    }
}
