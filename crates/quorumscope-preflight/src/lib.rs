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
                        "The result depends on which offers and trustlines this operation matches when the transaction is applied. The active freeze set has {} key(s), so the outcome cannot be confirmed before apply.",
                        frozen.len()
                    ),
                });
            }
            Concern::PathHops { path } if frozen_set_present => {
                findings.push(PreflightFinding {
                    status: PreflightStatus::ApplyTimeRisk,
                    confidence: PreflightConfidence::Conditional,
                    implicated_keys: vec![],
                    protocol_path: path.clone(),
                    explanation: format!(
                        "The ledger entries touched along the payment path are chosen during apply. The active freeze set has {} key(s), so the affected state is known only at apply time.",
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
