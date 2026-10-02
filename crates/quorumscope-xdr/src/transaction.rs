use stellar_xdr::{TransactionEnvelope, ReadXdr, WriteXdr};
use crate::error::XdrError;
use crate::codec::default_limits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTransactionEnvelope {
    pub envelope: TransactionEnvelope,
    pub canonical_xdr: Vec<u8>,
}

pub fn decode_transaction_envelope(bytes: &[u8]) -> Result<DecodedTransactionEnvelope, XdrError> {
    let envelope = TransactionEnvelope::from_xdr(bytes, default_limits())
        .map_err(XdrError::Decode)?;
        
    let canonical_xdr = envelope.to_xdr(default_limits())
        .map_err(XdrError::Decode)?;

    Ok(DecodedTransactionEnvelope {
        envelope,
        canonical_xdr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::{TransactionV1Envelope, Transaction, AccountId, PublicKey, Uint256, SequenceNumber, Preconditions, Memo, TransactionExt};
    use stellar_xdr::VecM;

    #[test]
    fn test_decode_transaction_envelope() {
        let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: Transaction {
                source_account: AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([0; 32]))).into(),
                fee: 100,
                seq_num: SequenceNumber(1),
                cond: Preconditions::None,
                memo: Memo::None,
                operations: VecM::try_from(vec![]).unwrap(),
                ext: TransactionExt::V0,
            },
            signatures: VecM::try_from(vec![]).unwrap(),
        });

        let bytes = envelope.to_xdr(default_limits()).unwrap();
        let decoded = decode_transaction_envelope(&bytes).unwrap();

        assert_eq!(decoded.envelope, envelope);
        assert_eq!(decoded.canonical_xdr, bytes);
    }
}
