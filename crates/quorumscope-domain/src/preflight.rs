use std::fmt;
use crate::network::NetworkName;
use crate::ledger::{LedgerSequence, LedgerCloseTime};
use crate::evidence::DataFreshness;
use crate::freeze::FrozenKeyId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PreflightStatus {
    Clear,
    StateUnavailable,
    UnsupportedAnalysis,
    InvalidInput,
    DexConditional,
    ApplyTimeRisk,
    AllowedByBypass,
    BlockedValidation,
}

impl fmt::Display for PreflightStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreflightStatus::Clear => write!(f, "clear"),
            PreflightStatus::StateUnavailable => write!(f, "state_unavailable"),
            PreflightStatus::UnsupportedAnalysis => write!(f, "unsupported_analysis"),
            PreflightStatus::InvalidInput => write!(f, "invalid_input"),
            PreflightStatus::DexConditional => write!(f, "dex_conditional"),
            PreflightStatus::ApplyTimeRisk => write!(f, "apply_time_risk"),
            PreflightStatus::AllowedByBypass => write!(f, "allowed_by_bypass"),
            PreflightStatus::BlockedValidation => write!(f, "blocked_validation"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightConfidence {
    Deterministic,
    Conditional,
    InsufficientInformation,
}

impl fmt::Display for PreflightConfidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreflightConfidence::Deterministic => write!(f, "deterministic"),
            PreflightConfidence::Conditional => write!(f, "conditional"),
            PreflightConfidence::InsufficientInformation => write!(f, "insufficient_information"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightFinding {
    pub status: PreflightStatus,
    pub confidence: PreflightConfidence,
    pub implicated_keys: Vec<FrozenKeyId>,
    pub protocol_path: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightResult {
    pub network: NetworkName,
    pub freeze_ledger: LedgerSequence,
    pub freeze_close_time: Option<LedgerCloseTime>,
    pub freshness: DataFreshness,
    pub transaction_hash: String, // Hex encoded transaction content hash
    pub is_bypassed: bool,
    pub status: PreflightStatus,
    pub confidence: PreflightConfidence,
    pub findings: Vec<PreflightFinding>,
}

impl PreflightResult {
    /// Deterministically derive the top-level status from a set of findings.
    /// The lowest enum value (which corresponds to the highest priority based on Ord derived)
    /// might be used, but blocked_validation should take precedence over clear.
    /// Let's define the precedence explicitly.
    pub fn derive_status(findings: &[PreflightFinding], is_bypassed: bool) -> PreflightStatus {
        if is_bypassed {
            return PreflightStatus::AllowedByBypass;
        }
        
        if findings.is_empty() {
            return PreflightStatus::Clear;
        }

        // Higher precedence -> wins
        // BlockedValidation > ApplyTimeRisk > DexConditional > UnsupportedAnalysis > InvalidInput > StateUnavailable > Clear
        let mut highest = PreflightStatus::Clear;
        for finding in findings {
            if finding.status > highest {
                highest = finding.status;
            }
        }
        highest
    }
    
    pub fn derive_confidence(findings: &[PreflightFinding]) -> PreflightConfidence {
        if findings.is_empty() {
            return PreflightConfidence::Deterministic;
        }
        
        let mut lowest = PreflightConfidence::Deterministic;
        for finding in findings {
            match finding.confidence {
                PreflightConfidence::InsufficientInformation => {
                    return PreflightConfidence::InsufficientInformation;
                }
                PreflightConfidence::Conditional => {
                    lowest = PreflightConfidence::Conditional;
                }
                PreflightConfidence::Deterministic => {}
            }
        }
        lowest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_display() {
        assert_eq!(PreflightStatus::Clear.to_string(), "clear");
        assert_eq!(PreflightStatus::BlockedValidation.to_string(), "blocked_validation");
    }

    #[test]
    fn test_derive_status() {
        assert_eq!(PreflightResult::derive_status(&[], true), PreflightStatus::AllowedByBypass);
        assert_eq!(PreflightResult::derive_status(&[], false), PreflightStatus::Clear);

        let findings = vec![
            PreflightFinding {
                status: PreflightStatus::DexConditional,
                confidence: PreflightConfidence::Conditional,
                implicated_keys: vec![],
                protocol_path: "".to_string(),
                explanation: "".to_string(),
            },
            PreflightFinding {
                status: PreflightStatus::BlockedValidation,
                confidence: PreflightConfidence::Deterministic,
                implicated_keys: vec![],
                protocol_path: "".to_string(),
                explanation: "".to_string(),
            }
        ];
        
        assert_eq!(PreflightResult::derive_status(&findings, false), PreflightStatus::BlockedValidation);
    }
}
