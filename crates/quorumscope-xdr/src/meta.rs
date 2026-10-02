use stellar_xdr::{TransactionMeta, LedgerEntryChange};

pub fn unzip_meta(meta: &TransactionMeta) -> Vec<&LedgerEntryChange> {
    let mut changes = Vec::new();
    
    match meta {
        TransactionMeta::V0(m) => {
            for op in m.iter() {
                changes.extend(op.changes.0.iter());
            }
        }
        TransactionMeta::V1(m) => {
            changes.extend(m.tx_changes.0.iter());
            for op in m.operations.iter() {
                changes.extend(op.changes.0.iter());
            }
        }
        TransactionMeta::V2(m) => {
            changes.extend(m.tx_changes_before.0.iter());
            for op in m.operations.iter() {
                changes.extend(op.changes.0.iter());
            }
            changes.extend(m.tx_changes_after.0.iter());
        }
        TransactionMeta::V3(m) => {
            changes.extend(m.tx_changes_before.0.iter());
            for op in m.operations.iter() {
                changes.extend(op.changes.0.iter());
            }
            changes.extend(m.tx_changes_after.0.iter());
        }
        TransactionMeta::V4(m) => {
            changes.extend(m.tx_changes_before.0.iter());
            for op in m.operations.iter() {
                changes.extend(op.changes.0.iter());
            }
            changes.extend(m.tx_changes_after.0.iter());
        }
    }
    
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use stellar_xdr::{TransactionMetaV3, ExtensionPoint, VecM, LedgerEntryChanges};

    #[test]
    fn test_unzip_meta_v3() {
        let meta_v3 = TransactionMetaV3 {
            ext: ExtensionPoint::V0,
            tx_changes_before: LedgerEntryChanges(VecM::try_from(vec![]).unwrap()),
            operations: VecM::try_from(vec![]).unwrap(),
            tx_changes_after: LedgerEntryChanges(VecM::try_from(vec![]).unwrap()),
            soroban_meta: None,
        };
        let meta = TransactionMeta::V3(meta_v3);
        let changes = unzip_meta(&meta);
        assert!(changes.is_empty());
    }
}
