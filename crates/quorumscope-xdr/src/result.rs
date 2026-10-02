use stellar_xdr::{TransactionResult, TransactionResultResult, InnerTransactionResult, InnerTransactionResultResult, OperationResult, VecM, ReadXdr, WriteXdr};
use crate::error::XdrError;
use crate::codec::default_limits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedTransactionResult {
    pub result: TransactionResult,
    pub canonical_xdr: Vec<u8>,
}

pub fn decode_transaction_result(bytes: &[u8]) -> Result<DecodedTransactionResult, XdrError> {
    let result = TransactionResult::from_xdr(bytes, default_limits())
        .map_err(XdrError::Decode)?;
        
    let canonical_xdr = result.to_xdr(default_limits())
        .map_err(XdrError::Decode)?;

    Ok(DecodedTransactionResult {
        result,
        canonical_xdr,
    })
}

pub fn get_operation_results(result: &TransactionResult) -> Option<&VecM<OperationResult>> {
    match &result.result {
        TransactionResultResult::TxSuccess(s) => Some(s),
        TransactionResultResult::TxFailed(f) => Some(f),
        TransactionResultResult::TxFeeBumpInnerSuccess(p) => {
            match &p.result.result {
                InnerTransactionResultResult::TxSuccess(s) => Some(s),
                InnerTransactionResultResult::TxFailed(f) => Some(f),
                _ => None,
            }
        },
        TransactionResultResult::TxFeeBumpInnerFailed(p) => {
            match &p.result.result {
                InnerTransactionResultResult::TxSuccess(s) => Some(s),
                InnerTransactionResultResult::TxFailed(f) => Some(f),
                _ => None,
            }
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::{TransactionResult, TransactionResultResult, TransactionResultExt};

    #[test]
    fn test_unwrap_inner_result_tx_success() {
        let res = TransactionResult {
            fee_charged: 100,
            result: TransactionResultResult::TxSuccess(VecM::try_from(vec![]).unwrap()),
            ext: TransactionResultExt::V0,
        };
        
        let op_results = get_operation_results(&res);
        assert!(op_results.is_some());
    }
}
