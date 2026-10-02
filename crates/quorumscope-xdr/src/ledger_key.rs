use crate::codec::default_limits;
use crate::error::XdrError;
use quorumscope_domain::freeze::{DecodedFrozenKey, FrozenKeyKind};
use serde_json::json;
use stellar_xdr::{LedgerKey, TrustLineAsset, WriteXdr};

pub fn decode_ledger_key(key: &LedgerKey) -> Result<DecodedFrozenKey, XdrError> {
    let (kind, decoded_json) = match key {
        LedgerKey::Account(a) => {
            let account_id = a.account_id.to_string();
            (
                FrozenKeyKind::Account,
                json!({ "account_id": account_id }).to_string(),
            )
        }
        LedgerKey::Trustline(t) => {
            let account_id = t.account_id.to_string();

            // Reject pool share trustlines
            if let TrustLineAsset::PoolShare(_) = t.asset {
                return Err(XdrError::UnsupportedLedgerKey);
            }

            // Reject issuer trustlines
            let is_issuer = match &t.asset {
                TrustLineAsset::Native => false, // Cannot trustline native, but usually validated elsewhere. We just pass it through or it will be rejected later if protocol forbids.
                TrustLineAsset::CreditAlphanum4(c) => c.issuer == t.account_id,
                TrustLineAsset::CreditAlphanum12(c) => c.issuer == t.account_id,
                _ => false,
            };
            if is_issuer {
                return Err(XdrError::UnsupportedLedgerKey);
            }

            let asset = format!("{:?}", t.asset);
            (
                FrozenKeyKind::Trustline,
                json!({ "account_id": account_id, "asset": asset }).to_string(),
            )
        }
        LedgerKey::ContractData(d) => {
            let contract = match &d.contract {
                stellar_xdr::ScAddress::Account(a) => format!("Account({})", a),
                stellar_xdr::ScAddress::Contract(c) => hex::encode(&c.0),
                other => format!("{:?}", other),
            };
            let key_val = format!("{:?}", d.key);
            let durability = format!("{:?}", d.durability);
            (
                FrozenKeyKind::ContractData,
                json!({ "contract": contract, "key": key_val, "durability": durability })
                    .to_string(),
            )
        }
        LedgerKey::ContractCode(c) => {
            let hash = hex::encode(c.hash.0);
            (
                FrozenKeyKind::ContractCode,
                json!({ "hash": hash }).to_string(),
            )
        }
        _ => return Err(XdrError::UnsupportedLedgerKey),
    };

    let canonical_xdr = key.to_xdr(default_limits()).map_err(XdrError::Decode)?;

    Ok(DecodedFrozenKey {
        kind,
        decoded_json,
        canonical_xdr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::StringM;
    use stellar_xdr::{
        AccountId, AlphaNum4, AssetCode4, LedgerKeyAccount, LedgerKeyData, LedgerKeyTrustLine,
        PublicKey, TrustLineAsset, Uint256,
    };

    #[test]
    fn test_decode_account_key() {
        let pk = PublicKey::PublicKeyTypeEd25519(Uint256([0; 32]));
        let account_id = AccountId(pk);
        let key = LedgerKey::Account(LedgerKeyAccount { account_id });

        let decoded = decode_ledger_key(&key).unwrap();
        assert_eq!(decoded.kind, FrozenKeyKind::Account);
        assert!(decoded.decoded_json.contains("account_id"));
    }

    #[test]
    fn test_reject_unsupported_key() {
        let key = LedgerKey::Data(LedgerKeyData {
            account_id: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([0; 32]))),
            data_name: stellar_xdr::String64(StringM::try_from(vec![]).unwrap()),
        });
        assert!(matches!(
            decode_ledger_key(&key),
            Err(XdrError::UnsupportedLedgerKey)
        ));
    }

    #[test]
    fn test_reject_pool_share_trustline() {
        let key = LedgerKey::Trustline(LedgerKeyTrustLine {
            account_id: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([0; 32]))),
            asset: TrustLineAsset::PoolShare(stellar_xdr::PoolId(stellar_xdr::Hash([0; 32]))),
        });
        assert!(matches!(
            decode_ledger_key(&key),
            Err(XdrError::UnsupportedLedgerKey)
        ));
    }

    #[test]
    fn test_reject_issuer_trustline() {
        let account_id = AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([0; 32])));
        let key = LedgerKey::Trustline(LedgerKeyTrustLine {
            account_id: account_id.clone(),
            asset: TrustLineAsset::CreditAlphanum4(AlphaNum4 {
                asset_code: AssetCode4([b'A', b'B', b'C', 0]),
                issuer: account_id,
            }),
        });
        assert!(matches!(
            decode_ledger_key(&key),
            Err(XdrError::UnsupportedLedgerKey)
        ));
    }
}
