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

#[cfg(test)]
mod tests {
    use super::*;
    use quorumscope_domain::freeze::FrozenKeyKind;

    fn key(byte: u8) -> DecodedFrozenKey {
        DecodedFrozenKey {
            kind: FrozenKeyKind::Account,
            decoded_json: "{}".into(),
            canonical_xdr: vec![byte; 4],
        }
    }

    fn bypass(byte: u8) -> DecodedBypassTransaction {
        DecodedBypassTransaction {
            hash: hex::encode([byte; 32]),
            decoded_json: serde_json::Value::Null,
            canonical_xdr: vec![byte; 32],
        }
    }

    fn ledger() -> LedgerSequence {
        LedgerSequence::new(10).unwrap()
    }

    #[test]
    fn new_keys_freeze_and_missing_keys_unfreeze() {
        let network = NetworkId::new();
        let kept = FrozenKeyId::new(FreezeStateMachine::hash_xdr(&key(1).canonical_xdr));
        let gone = FrozenKeyId::new([9; 32]);
        let current: HashSet<_> = [kept, gone].into_iter().collect();
        let changes = FreezeStateMachine::compute_freeze_changes(
            network,
            ledger(),
            &current,
            &[key(1), key(2)],
        );
        let frozen: Vec<_> = changes
            .iter()
            .filter(|c| c.action == FreezeAction::Freeze)
            .collect();
        let unfrozen: Vec<_> = changes
            .iter()
            .filter(|c| c.action == FreezeAction::Unfreeze)
            .collect();
        assert_eq!(frozen.len(), 1);
        assert_eq!(
            frozen[0].key_hash,
            FrozenKeyId::new(FreezeStateMachine::hash_xdr(&key(2).canonical_xdr))
        );
        assert_eq!(unfrozen.len(), 1);
        assert_eq!(unfrozen[0].key_hash, gone);
    }

    #[test]
    fn observing_the_same_set_produces_no_changes() {
        let id = FrozenKeyId::new(FreezeStateMachine::hash_xdr(&key(1).canonical_xdr));
        let current: HashSet<_> = [id].into_iter().collect();
        let changes = FreezeStateMachine::compute_freeze_changes(
            NetworkId::new(),
            ledger(),
            &current,
            &[key(1)],
        );
        assert!(changes.is_empty());
    }

    #[test]
    fn bypasses_are_added_and_removed_by_hash() {
        let network = NetworkId::new();
        let old = BypassHash::new([7; 32]);
        let current: HashSet<_> = [old].into_iter().collect();
        let changes =
            FreezeStateMachine::compute_bypass_changes(network, ledger(), &current, &[bypass(8)]);
        assert_eq!(changes.len(), 2);
        assert!(
            changes
                .iter()
                .any(|c| c.action == BypassAction::Add && c.bypass_hash == BypassHash::new([8; 32]))
        );
        assert!(
            changes
                .iter()
                .any(|c| c.action == BypassAction::Remove && c.bypass_hash == old)
        );
        let none = FreezeStateMachine::compute_bypass_changes(
            network,
            ledger(),
            &[BypassHash::new([8; 32])].into_iter().collect(),
            &[bypass(8)],
        );
        assert!(none.is_empty());
    }
}
