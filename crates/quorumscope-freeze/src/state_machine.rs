use quorumscope_domain::bypass::{
    BypassAction, BypassChange, BypassChangeResult, BypassHash, DecodedBypassTransaction,
};
use quorumscope_domain::freeze::{
    DecodedFrozenKey, FreezeAction, FreezeChange, FreezeChangeResult, FrozenKeyId,
};
use quorumscope_domain::ledger::LedgerSequence;
use quorumscope_domain::network::NetworkId;
use std::collections::HashSet;
use std::time::SystemTime;

pub struct FreezeStateMachine;

impl FreezeStateMachine {
    /// Compares currently known active keys with the newly observed keys from a ledger.
    /// Returns a list of changes (Freeze or Unfreeze events).
    pub fn compute_freeze_changes(
        network_id: NetworkId,
        ledger: LedgerSequence,
        current_active: &HashSet<FrozenKeyId>,
        observed_keys: &[DecodedFrozenKey],
    ) -> Vec<FreezeChange> {
        let mut changes = Vec::new();
        let now = SystemTime::now();

        let mut observed_hashes = HashSet::new();

        // Calculate additions
        for key in observed_keys {
            let key_hash = Self::hash_xdr(&key.canonical_xdr);
            let id = FrozenKeyId::new(key_hash);
            observed_hashes.insert(id);

            if !current_active.contains(&id) {
                changes.push(FreezeChange {
                    network_id,
                    ledger_sequence: ledger,
                    key_hash: id,
                    action: FreezeAction::Freeze,
                    result: FreezeChangeResult::Changed,
                    evidence_ref: None,
                    created_at: now,
                });
            }
        }

        // Calculate removals
        for id in current_active {
            if !observed_hashes.contains(id) {
                changes.push(FreezeChange {
                    network_id,
                    ledger_sequence: ledger,
                    key_hash: *id,
                    action: FreezeAction::Unfreeze,
                    result: FreezeChangeResult::Changed,
                    evidence_ref: None,
                    created_at: now,
                });
            }
        }

        changes
    }

    /// Computes changes for bypass transactions similarly.
    pub fn compute_bypass_changes(
        network_id: NetworkId,
        ledger: LedgerSequence,
        current_active: &HashSet<BypassHash>,
        observed_bypasses: &[DecodedBypassTransaction],
    ) -> Vec<BypassChange> {
        let mut changes = Vec::new();
        let now = SystemTime::now();

        let mut observed_hashes = HashSet::new();

        // Additions
        for bypass in observed_bypasses {
            // The bypass hash is a hex string, we parse it to 32 bytes
            let mut byte_hash = [0u8; 32];
            if let Ok(bytes) = hex::decode(&bypass.hash)
                && let Ok(arr) = <[u8; 32]>::try_from(bytes.as_slice())
            {
                byte_hash = arr;
            }
            let bhash = BypassHash::new(byte_hash);
            observed_hashes.insert(bhash);

            if !current_active.contains(&bhash) {
                changes.push(BypassChange {
                    network_id,
                    ledger_sequence: ledger,
                    bypass_hash: bhash,
                    action: BypassAction::Add,
                    result: BypassChangeResult::Changed,
                    evidence_ref: None,
                    created_at: now,
                });
            }
        }

        // Removals
        for hash in current_active {
            if !observed_hashes.contains(hash) {
                changes.push(BypassChange {
                    network_id,
                    ledger_sequence: ledger,
                    bypass_hash: *hash,
                    action: BypassAction::Remove,
                    result: BypassChangeResult::Changed,
                    evidence_ref: None,
                    created_at: now,
                });
            }
        }

        changes
    }

    pub fn hash_xdr(xdr: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(xdr);
        let result = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&result);
        arr
    }
}
