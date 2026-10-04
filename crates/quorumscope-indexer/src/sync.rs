use crate::error::IndexerError;
use quorumscope_domain::bypass::DecodedBypassTransaction;
use quorumscope_domain::bypass::{BypassChange, BypassHash};
use quorumscope_domain::freeze::{DecodedFrozenKey, FreezeChange, FrozenKeyId};
use quorumscope_domain::ledger::LedgerSequence;
use quorumscope_domain::network::NetworkId;
use quorumscope_freeze::state_machine::FreezeStateMachine;
use quorumscope_rpc::client::RpcClient;
use quorumscope_rpc::get_ledger_entries::{GetLedgerEntriesRequest, GetLedgerEntriesResponse};
use quorumscope_rpc::latest_ledger::{GetLatestLedgerRequest, GetLatestLedgerResponse};
use quorumscope_storage::indexer::IndexerRepository;
use quorumscope_storage::poll::{PollObservation, PollRepository, SnapshotRecord};
use quorumscope_xdr::codec::{decode_xdr_base64, encode_xdr_base64};
use quorumscope_xdr::config::{freeze_bypass_txs_key, frozen_ledger_keys_key};
use quorumscope_xdr::ledger_key::decode_ledger_key;
use std::time::Duration;
use stellar_xdr::{ConfigSettingEntry, LedgerEntryData, LedgerKey, ReadXdr};
use tokio::time::MissedTickBehavior;
use uuid::Uuid;

const STREAM: &str = "cap77_config";

pub struct SyncTask {
    client: RpcClient,
    repo: IndexerRepository,
    poll: PollRepository,
    network_id: NetworkId,
}

impl SyncTask {
    pub fn new(client: RpcClient, repo: IndexerRepository, network_id: NetworkId) -> Self {
        let poll = PollRepository::new(repo.pool());
        Self {
            client,
            repo,
            poll,
            network_id,
        }
    }

    async fn latest_network(&self, floor: u32) -> (i64, Option<i32>) {
        let request = GetLatestLedgerRequest::new(2);
        match self
            .client
            .send_request::<_, GetLatestLedgerResponse>(&request)
            .await
        {
            Ok(GetLatestLedgerResponse {
                result: Some(r), ..
            }) => (
                i64::from(r.sequence.max(floor)),
                i32::try_from(r.protocol_version).ok(),
            ),
            Ok(other) => {
                tracing::warn!(error = ?other.error, "getLatestLedger returned no result");
                (i64::from(floor), None)
            }
            Err(e) => {
                tracing::warn!(error = %e, "getLatestLedger failed");
                (i64::from(floor), None)
            }
        }
    }

    pub async fn run_once(&self) -> Result<(), IndexerError> {
        let pipeline = |e: &dyn std::fmt::Display| IndexerError::Pipeline(e.to_string());
        let keys = [frozen_ledger_keys_key(), freeze_bypass_txs_key()]
            .iter()
            .map(encode_xdr_base64)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| pipeline(&e))?;
        let req = GetLedgerEntriesRequest::new(1, keys);
        let res: GetLedgerEntriesResponse = self
            .client
            .send_request(&req)
            .await
            .map_err(|e| pipeline(&e))?;
        if let Some(err) = res.error {
            return Err(IndexerError::Pipeline(format!("RPC error: {err:?}")));
        }
        let result = res
            .result
            .ok_or_else(|| IndexerError::Pipeline("No result in RPC response".into()))?;
        let ledger = i64::from(result.latest_ledger);
        let (latest_network_ledger, protocol_version) =
            self.latest_network(result.latest_ledger).await;

        if let Some(last) = self
            .repo
            .get_checkpoint(self.network_id, STREAM)
            .await
            .map_err(|e| pipeline(&e))?
            && last > ledger
        {
            tracing::info!(
                ledger,
                checkpoint = last,
                "RPC ledger is behind the checkpoint. Skipping."
            );
            self.poll
                .record_network_observation(
                    self.network_id,
                    latest_network_ledger,
                    protocol_version,
                )
                .await
                .map_err(|e| pipeline(&e))?;
            return Ok(());
        }

        let sequence = LedgerSequence::new(u32::try_from(ledger).map_err(|e| pipeline(&e))?)
            .map_err(|e| pipeline(&e))?;

        let mut snapshots = Vec::new();
        let mut observed_keys: Vec<(FrozenKeyId, DecodedFrozenKey)> = Vec::new();
        let mut observed_bypasses: Vec<BypassHash> = Vec::new();
        let mut frozen_ref = None;
        let mut bypass_ref = None;

        for entry in result.entries {
            let data: LedgerEntryData = decode_xdr_base64(&entry.xdr).map_err(|e| pipeline(&e))?;
            let raw =
                quorumscope_xdr::codec::decode_base64(&entry.xdr).map_err(|e| pipeline(&e))?;
            let LedgerEntryData::ConfigSetting(setting) = data else {
                continue;
            };
            match &setting {
                ConfigSettingEntry::FrozenLedgerKeys(frozen) => {
                    for encoded in frozen.keys.as_vec() {
                        let key = match LedgerKey::from_xdr(
                            encoded.0.as_slice(),
                            quorumscope_xdr::codec::default_limits(),
                        ) {
                            Ok(k) => k,
                            Err(e) => {
                                tracing::warn!(error = %e, "Skipping undecodable frozen key");
                                continue;
                            }
                        };
                        match decode_ledger_key(&key) {
                            Ok(decoded) => {
                                let id = FrozenKeyId::new(FreezeStateMachine::hash_xdr(
                                    &decoded.canonical_xdr,
                                ));
                                observed_keys.push((id, decoded));
                            }
                            Err(e) => tracing::warn!(error = %e, "Skipping unsupported frozen key"),
                        }
                    }
                    let snap = SnapshotRecord {
                        id: Uuid::new_v4(),
                        setting_id: "FrozenLedgerKeys",
                        raw_xdr: raw,
                        parsed_json: serde_json::json!({ "keys_count": frozen.keys.len() }),
                    };
                    frozen_ref = Some(snap.evidence_ref());
                    snapshots.push(snap);
                }
                ConfigSettingEntry::FreezeBypassTxs(txs) => {
                    for hash in txs.tx_hashes.as_vec() {
                        observed_bypasses.push(BypassHash::new(hash.0));
                    }
                    let snap = SnapshotRecord {
                        id: Uuid::new_v4(),
                        setting_id: "FreezeBypassTxs",
                        raw_xdr: raw,
                        parsed_json: serde_json::json!({ "tx_hashes_count": txs.tx_hashes.len() }),
                    };
                    bypass_ref = Some(snap.evidence_ref());
                    snapshots.push(snap);
                }
                _ => {}
            }
        }

        let current_frozen = self
            .repo
            .get_current_frozen_keys(self.network_id)
            .await
            .map_err(|e| pipeline(&e))?;
        let current_bypasses = self
            .repo
            .get_current_bypasses(self.network_id)
            .await
            .map_err(|e| pipeline(&e))?;

        let decoded: Vec<DecodedFrozenKey> = observed_keys.iter().map(|(_, k)| k.clone()).collect();
        let mut freeze_changes: Vec<FreezeChange> = FreezeStateMachine::compute_freeze_changes(
            self.network_id,
            sequence,
            &current_frozen,
            &decoded,
        );
        for change in &mut freeze_changes {
            change.evidence_ref = frozen_ref.clone();
        }
        let observed_bypass_txs: Vec<DecodedBypassTransaction> = observed_bypasses
            .iter()
            .map(|h| DecodedBypassTransaction {
                hash: h.to_string(),
                decoded_json: serde_json::Value::Null,
                canonical_xdr: h.as_bytes().to_vec(),
            })
            .collect();
        let mut bypass_changes: Vec<BypassChange> = FreezeStateMachine::compute_bypass_changes(
            self.network_id,
            sequence,
            &current_bypasses,
            &observed_bypass_txs,
        );
        for change in &mut bypass_changes {
            change.evidence_ref = bypass_ref.clone();
        }

        let freeze_count = freeze_changes.len();
        let bypass_count = bypass_changes.len();
        let outcome = self
            .poll
            .apply(&PollObservation {
                network_id: self.network_id,
                ledger,
                latest_network_ledger,
                protocol_version,
                snapshots,
                observed_keys,
                observed_bypasses,
                freeze_changes,
                bypass_changes,
            })
            .await
            .map_err(|e| pipeline(&e))?;

        tracing::info!(
            ledger,
            freeze_changes = freeze_count,
            bypass_changes = bypass_count,
            incident_opened = outcome.incident_opened,
            incident_closed = outcome.incident_closed,
            reconciled = outcome.reconciled,
            "Processed ledger"
        );
        if !outcome.reconciled {
            return Err(IndexerError::Pipeline(
                "stored freeze state does not match the configuration read from the network".into(),
            ));
        }
        Ok(())
    }

    pub async fn run_watch(&self, interval: Duration) -> Result<(), IndexerError> {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;
        let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .map_err(|e| IndexerError::Pipeline(e.to_string()))?;
        let mut ticker = tokio::time::interval(interval);
        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        tracing::info!("Starting watch mode. Polling interval: {:?}", interval);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
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
