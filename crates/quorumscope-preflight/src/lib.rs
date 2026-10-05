//! Transaction preflight against the active CAP-77 freeze set.
//!
//! The analysis lists the ledger keys a transaction envelope names, compares
//! their hashes with the stored freeze set, and reports what it could not
//! determine. It does not simulate transaction application.

mod touches;

use quorumscope_domain::bypass::BypassHash;
use quorumscope_domain::freeze::FrozenKeyId;
use quorumscope_domain::preflight::{PreflightConfidence, PreflightFinding, PreflightStatus};
use quorumscope_xdr::codec::default_limits;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use stellar_xdr::{
    Hash, LedgerKey, TransactionEnvelope, TransactionSignaturePayload,
    TransactionSignaturePayloadTaggedTransaction, WriteXdr,
};
use touches::{Concern, Touch, collect};

pub struct Analysis {
    /// Hex of the transaction content hash for the configured network.
    pub transaction_hash: String,
    pub bypassed: bool,
    pub findings: Vec<PreflightFinding>,
}

/// Hash of the transaction as the network signs it. For a fee bump envelope this is the
/// outer fee bump transaction.
pub fn content_hash(envelope: &TransactionEnvelope, passphrase: &str) -> BypassHash {
    let tagged = match envelope {
        TransactionEnvelope::TxV0(v0) => {
            TransactionSignaturePayloadTaggedTransaction::Tx(touches::v0_as_v1(&v0.tx))
        }
        TransactionEnvelope::Tx(v1) => {
            TransactionSignaturePayloadTaggedTransaction::Tx(v1.tx.clone())
        }
        TransactionEnvelope::TxFeeBump(fb) => {
            TransactionSignaturePayloadTaggedTransaction::TxFeeBump(fb.tx.clone())
        }
    };
    let payload = TransactionSignaturePayload {
        network_id: Hash(Sha256::digest(passphrase.as_bytes()).into()),
        tagged_transaction: tagged,
    };
    let bytes = payload
        .to_xdr(default_limits())
        .expect("a decoded transaction always re-encodes");
    BypassHash::new(Sha256::digest(bytes).into())
}

fn key_id(key: &LedgerKey) -> FrozenKeyId {
    let bytes = key
        .to_xdr(default_limits())
        .expect("a decoded ledger key always re-encodes");
    FrozenKeyId::new(Sha256::digest(bytes).into())
}

pub fn analyze(
    envelope: &TransactionEnvelope,
    passphrase: &str,
    frozen: &HashSet<FrozenKeyId>,
    bypasses: &HashSet<BypassHash>,
) -> Analysis {
    let hash = content_hash(envelope, passphrase);
    let mut findings = Vec::new();
    let mut seen = HashSet::new();
    let touched = collect(envelope);

    for Touch { key, path } in &touched.keys {
        let id = key_id(key);
        if frozen.contains(&id) && seen.insert((id, path.clone())) {
            findings.push(PreflightFinding {
                status: PreflightStatus::BlockedValidation,
                confidence: PreflightConfidence::Deterministic,
                implicated_keys: vec![id],
                protocol_path: path.clone(),
                explanation: format!(
                    "The transaction names a ledger key in the active freeze set ({path})."
                ),
            });
        }
    }

    let frozen_set_present = !frozen.is_empty();
    for concern in &touched.concerns {
        match concern {
            Concern::Dex { path } if frozen_set_present => {
                findings.push(PreflightFinding {
                    status: PreflightStatus::DexConditional,
                    confidence: PreflightConfidence::Conditional,
                    implicated_keys: vec![],
                    protocol_path: path.clone(),
                    explanation: format!(
                        "While matching offers, QuorumScope cannot tell whether an offer owner or trustline is frozen. The protocol removes such an offer without moving assets and keeps matching, so the transaction does not fail for that reason. A bypass does not apply at that stage. The active freeze set has {} key(s).",
                        frozen.len()
                    ),
                });
            }
            Concern::ApplyTime { path } if frozen_set_present => {
                findings.push(PreflightFinding {
                    status: PreflightStatus::ApplyTimeRisk,
                    confidence: PreflightConfidence::Conditional,
                    implicated_keys: vec![],
                    protocol_path: path.clone(),
                    explanation: format!(
                        "This operation names balances or pools by opaque identifier, so a frozen trustline or account is detected only when the transaction is applied, where it fails. A bypass does not apply at that stage. The active freeze set has {} key(s).",
                        frozen.len()
                    ),
                });
            }
            Concern::Unsupported { path, reason } => {
                findings.push(PreflightFinding {
                    status: PreflightStatus::UnsupportedAnalysis,
                    confidence: PreflightConfidence::InsufficientInformation,
                    implicated_keys: vec![],
                    protocol_path: path.clone(),
                    explanation: reason.clone(),
                });
            }
            _ => {}
        }
    }

    Analysis {
        transaction_hash: hash.to_string(),
        bypassed: bypasses.contains(&hash),
        findings,
    }
}

#[cfg(test)]
mod tests;
