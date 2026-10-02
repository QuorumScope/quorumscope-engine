pub mod network;
pub mod ledger;

#[cfg(test)]
mod tests {
    use super::network::{NetworkName, NetworkPassphrase};
    use super::ledger::{LedgerSequence, ProtocolVersion, LedgerHash};

    #[test]
    fn test_network_passphrase_validation() {
        let name = NetworkName::new("mainnet").unwrap();
        let passphrase = NetworkPassphrase::new("Public Global Stellar Network ; September 2015").unwrap();
        assert!(passphrase.validate_for_network(&name).is_ok());

        let invalid_passphrase = NetworkPassphrase::new("Test SDF Network ; September 2015").unwrap();
        assert!(invalid_passphrase.validate_for_network(&name).is_err());
    }

    #[test]
    fn test_custom_network_passphrase() {
        let name = NetworkName::new("my-custom-net").unwrap();
        let passphrase = NetworkPassphrase::new("Some custom passphrase").unwrap();
        assert!(passphrase.validate_for_network(&name).is_ok());
    }

    #[test]
    fn test_ledger_sequence_zero() {
        assert!(LedgerSequence::new(0).is_err());
        assert!(LedgerSequence::new(1).is_ok());
    }

    #[test]
    fn test_protocol_version_zero() {
        assert!(ProtocolVersion::new(0).is_err());
        assert!(ProtocolVersion::new(28).is_ok());
    }

    #[test]
    fn test_ledger_hash_length() {
        let valid = [0u8; 32];
        assert!(LedgerHash::from_slice(&valid).is_ok());
        
        let invalid = [0u8; 31];
        assert!(LedgerHash::from_slice(&invalid).is_err());
    }
}
pub mod evidence;
