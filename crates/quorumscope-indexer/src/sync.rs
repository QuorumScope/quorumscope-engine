use crate::error::IndexerError;
use quorumscope_domain::bypass::DecodedBypassTransaction;
use quorumscope_domain::freeze::{DecodedFrozenKey, FrozenKeyKind};
use quorumscope_domain::network::NetworkId;
use quorumscope_freeze::state_machine::FreezeStateMachine;
use quorumscope_rpc::client::RpcClient;
use quorumscope_rpc::get_ledger_entries::GetLedgerEntriesRequest;
use quorumscope_storage::indexer::IndexerRepository;
use quorumscope_xdr::codec::{decode_xdr_base64, encode_xdr_base64};
use quorumscope_xdr::config::{freeze_bypass_txs_key, frozen_ledger_keys_key};
use std::time::Duration;
use stellar_xdr::{ConfigSettingEntry, LedgerEntryData, ReadXdr};

pub struct SyncTask {
    client: RpcClient,
    repo: IndexerRepository,
    network_id: NetworkId,
}

impl SyncTask {
    pub fn new(client: RpcClient, repo: IndexerRepository, network_id: NetworkId) -> Self {
        Self {
            client,
            repo,
            network_id,
        }
    }

    pub async fn run_once(&self) -> Result<(), IndexerError> {
        let frozen_key = frozen_ledger_keys_key();
        let bypass_key = freeze_bypass_txs_key();

        let frozen_key_b64 =
            encode_xdr_base64(&frozen_key).map_err(|e| IndexerError::Pipeline(e.to_string()))?;
        let bypass_key_b64 =
            encode_xdr_base64(&bypass_key).map_err(|e| IndexerError::Pipeline(e.to_string()))?;

        let req = GetLedgerEntriesRequest::new(1, vec![frozen_key_b64, bypass_key_b64]);

        let res: quorumscope_rpc::get_ledger_entries::GetLedgerEntriesResponse = self
            .client
            .send_request(&req)
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;

        if let Some(err) = res.error {
            return Err(IndexerError::Pipeline(format!("RPC error: {:?}", err)));
        }

        let result = res
            .result
            .ok_or_else(|| IndexerError::Pipeline("No result in RPC response".into()))?;
        let ledger = result.latest_ledger as i64;

        // Check idempotency
        if let Some(last_ledger) = self
            .repo
            .get_checkpoint(self.network_id, "cap77_config")
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?
            && last_ledger >= ledger
        {
            tracing::info!("Ledger {} already processed. Skipping.", ledger);
            return Ok(());
        }

        let mut observed_frozen_keys = Vec::new();
        let mut observed_bypasses = Vec::new();

        for entry_res in result.entries {
            let data: LedgerEntryData = decode_xdr_base64(&entry_res.xdr)
                .map_err(|e| IndexerError::Pipeline(e.to_string()))?;

            if let LedgerEntryData::ConfigSetting(config_setting) = data {
                match &config_setting {
                    ConfigSettingEntry::FrozenLedgerKeys(keys) => {
                        let json = serde_json::json!({ "keys_count": keys.keys.len() });

                        // We must decode into DecodedFrozenKey
                        for encoded_key in keys.keys.as_vec() {
                            let xdr_bytes = encoded_key.0.as_slice();

                            let decoded_key = match stellar_xdr::LedgerKey::from_xdr(
                                xdr_bytes,
                                quorumscope_xdr::codec::default_limits(),
                            ) {
                                Ok(k) => k,
                                Err(_) => continue, // ignore decode errors here
                            };

                            let kind = match decoded_key {
                                stellar_xdr::LedgerKey::Account(_) => FrozenKeyKind::Account,
                                stellar_xdr::LedgerKey::Trustline(_) => FrozenKeyKind::Trustline,
                                stellar_xdr::LedgerKey::ContractData(_) => {
                                    FrozenKeyKind::ContractData
                                }
                                stellar_xdr::LedgerKey::ContractCode(_) => {
                                    FrozenKeyKind::ContractCode
                                }
                                _ => continue, // ignore unsupported
                            };

                            let canonical_xdr = xdr_bytes.to_vec();
                            let decoded_json =
                                serde_json::json!({ "type": kind.to_string() }).to_string();

                            observed_frozen_keys.push(DecodedFrozenKey {
                                kind,
                                decoded_json,
                                canonical_xdr,
                            });
                        }

                        // Store snapshot
                        self.repo
                            .store_config_snapshot(
                                self.network_id,
                                ledger,
                                "FrozenLedgerKeys",
                                &quorumscope_xdr::codec::decode_base64(&entry_res.xdr).unwrap(),
                                &json,
                            )
                            .await
                            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;
                    }
                    ConfigSettingEntry::FreezeBypassTxs(txs) => {
                        let json = serde_json::json!({ "tx_hashes_count": txs.tx_hashes.len() });

                        for tx in txs.tx_hashes.as_vec() {
                            let hex_hash = hex::encode(tx.0);
                            observed_bypasses.push(DecodedBypassTransaction {
                                hash: hex_hash,
                                decoded_json: serde_json::Value::Null, // Or better parsed if needed
                                canonical_xdr: tx.0.to_vec(),
                            });
                        }

                        // Store snapshot
                        self.repo
                            .store_config_snapshot(
                                self.network_id,
                                ledger,
                                "FreezeBypassTxs",
                                &quorumscope_xdr::codec::decode_base64(&entry_res.xdr).unwrap(),
                                &json,
                            )
                            .await
                            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;
                    }
                    _ => {}
                }
            }
        }

        let current_frozen = self
            .repo
            .get_current_frozen_keys(self.network_id)
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;
        let current_bypasses_db = self
            .repo
            .get_current_bypasses(self.network_id)
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;

        let freeze_changes = FreezeStateMachine::compute_freeze_changes(
            self.network_id,
            quorumscope_domain::ledger::LedgerSequence::new(ledger as u32).unwrap(),
            &current_frozen,
            &observed_frozen_keys,
        );

        let bypass_changes = FreezeStateMachine::compute_bypass_changes(
            self.network_id,
            quorumscope_domain::ledger::LedgerSequence::new(ledger as u32).unwrap(),
            &current_bypasses_db,
            &observed_bypasses,
        );

        self.repo
            .apply_freeze_changes(&freeze_changes)
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;
        self.repo
            .apply_bypass_changes(&bypass_changes)
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;
        self.repo
            .upsert_checkpoint(self.network_id, "cap77_config", ledger)
            .await
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;

        tracing::info!(
            "Processed ledger {}. Freeze changes: {}, Bypass changes: {}",
            ledger,
            freeze_changes.len(),
            bypass_changes.len()
        );

        Ok(())
    }

    pub async fn run_watch(&self, interval: Duration) -> Result<(), IndexerError> {
        let mut sigterm =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
        let mut sigint =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).unwrap();

        tracing::info!("Starting watch mode. Polling interval: {:?}", interval);

        loop {
            tokio::select! {
                _ = tokio::time::sleep(interval) => {
                    if let Err(e) = self.run_once().await {
                        tracing::error!("Sync error: {}", e);
                    }
                }
                _ = sigterm.recv() => {
                    tracing::info!("Received SIGTERM, shutting down gracefully.");
                    break;
                }
                _ = sigint.recv() => {
                    tracing::info!("Received SIGINT, shutting down gracefully.");
                    break;
                }
            }
        }
        Ok(())
    }
}
