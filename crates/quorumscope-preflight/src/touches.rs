use stellar_xdr::{
    AccountId, Asset, ChangeTrustAsset, LedgerKey, LedgerKeyAccount, LedgerKeyTrustLine,
    MuxedAccount, Operation, OperationBody, Preconditions, PublicKey, Transaction,
    TransactionEnvelope, TransactionExt, TransactionV0, TransactionV0Ext, TrustLineAsset,
};

pub struct Touch {
    pub key: LedgerKey,
    pub path: String,
}

pub enum Concern {
    Dex { path: String },
    PathHops { path: String },
    Unsupported { path: String, reason: String },
}

#[derive(Default)]
pub struct Touched {
    pub keys: Vec<Touch>,
    pub concerns: Vec<Concern>,
}

impl Touched {
    fn account(&mut self, id: &AccountId, path: String) {
        self.keys.push(Touch {
            key: LedgerKey::Account(LedgerKeyAccount {
                account_id: id.clone(),
            }),
            path,
        });
    }

    fn trustline(&mut self, id: &AccountId, line: Option<TrustLineAsset>, path: String) {
        if let Some(asset) = line {
            self.keys.push(Touch {
                key: LedgerKey::Trustline(LedgerKeyTrustLine {
                    account_id: id.clone(),
                    asset,
                }),
                path,
            });
        }
    }
}

pub fn v0_as_v1(tx: &TransactionV0) -> Transaction {
    let cond = match &tx.time_bounds {
        Some(bounds) => Preconditions::Time(bounds.clone()),
        None => Preconditions::None,
    };
    let TransactionV0Ext::V0 = tx.ext;
    Transaction {
        source_account: MuxedAccount::Ed25519(tx.source_account_ed25519.clone()),
        fee: tx.fee,
        seq_num: tx.seq_num.clone(),
        cond,
        memo: tx.memo.clone(),
        operations: tx.operations.clone(),
        ext: TransactionExt::V0,
    }
}

fn account_id(muxed: &MuxedAccount) -> AccountId {
    match muxed {
        MuxedAccount::Ed25519(key) => AccountId(PublicKey::PublicKeyTypeEd25519(key.clone())),
        MuxedAccount::MuxedEd25519(m) => {
            AccountId(PublicKey::PublicKeyTypeEd25519(m.ed25519.clone()))
        }
    }
}

/// Trustline for a credit asset. The native asset and an issuer holding its own asset have none.
fn line(holder: &AccountId, asset: &Asset) -> Option<TrustLineAsset> {
    match asset {
        Asset::Native => None,
        Asset::CreditAlphanum4(a) if a.issuer != *holder => {
            Some(TrustLineAsset::CreditAlphanum4(a.clone()))
        }
        Asset::CreditAlphanum12(a) if a.issuer != *holder => {
            Some(TrustLineAsset::CreditAlphanum12(a.clone()))
        }
        _ => None,
    }
}

fn change_trust_line(holder: &AccountId, asset: &ChangeTrustAsset) -> Option<TrustLineAsset> {
    match asset {
        ChangeTrustAsset::Native | ChangeTrustAsset::PoolShare(_) => None,
        ChangeTrustAsset::CreditAlphanum4(a) if a.issuer != *holder => {
            Some(TrustLineAsset::CreditAlphanum4(a.clone()))
        }
        ChangeTrustAsset::CreditAlphanum12(a) if a.issuer != *holder => {
            Some(TrustLineAsset::CreditAlphanum12(a.clone()))
        }
        _ => None,
    }
}

pub fn collect(envelope: &TransactionEnvelope) -> Touched {
    let mut out = Touched::default();
    match envelope {
        TransactionEnvelope::TxV0(v0) => transaction(&mut out, &v0_as_v1(&v0.tx)),
        TransactionEnvelope::Tx(v1) => transaction(&mut out, &v1.tx),
        TransactionEnvelope::TxFeeBump(fb) => {
            out.account(&account_id(&fb.tx.fee_source), "fee source account".into());
            let stellar_xdr::FeeBumpTransactionInnerTx::Tx(inner) = &fb.tx.inner_tx;
            transaction(&mut out, &inner.tx);
        }
    }
    out
}

fn transaction(out: &mut Touched, tx: &Transaction) {
    let source = account_id(&tx.source_account);
    out.account(&source, "transaction source account".into());
    for (index, op) in tx.operations.iter().enumerate() {
        operation(out, index, op, &source);
    }
    if let TransactionExt::V1(data) = &tx.ext {
        for key in data.resources.footprint.read_only.iter() {
            out.keys.push(Touch {
                key: key.clone(),
                path: "Soroban footprint (read-only)".into(),
            });
        }
        for key in data.resources.footprint.read_write.iter() {
            out.keys.push(Touch {
                key: key.clone(),
                path: "Soroban footprint (read-write)".into(),
            });
        }
    }
}

fn operation(out: &mut Touched, index: usize, op: &Operation, tx_source: &AccountId) {
    let source = op
        .source_account
        .as_ref()
        .map_or_else(|| tx_source.clone(), account_id);
    let name = op.body.name();
    let at = |role: &str| format!("operation {index} ({name}): {role}");
    if op.source_account.is_some() {
        out.account(&source, at("operation source account"));
    }
    match &op.body {
        OperationBody::CreateAccount(o) => out.account(&o.destination, at("new account")),
        OperationBody::Payment(o) => {
            let dest = account_id(&o.destination);
            out.trustline(&source, line(&source, &o.asset), at("sender trustline"));
            out.account(&dest, at("destination account"));
            out.trustline(&dest, line(&dest, &o.asset), at("destination trustline"));
        }
        OperationBody::PathPaymentStrictReceive(o) => {
            let dest = account_id(&o.destination);
            out.trustline(&source, line(&source, &o.send_asset), at("sender trustline"));
            out.account(&dest, at("destination account"));
            out.trustline(&dest, line(&dest, &o.dest_asset), at("destination trustline"));
            out.concerns.push(Concern::PathHops { path: at("payment path") });
        }
        OperationBody::PathPaymentStrictSend(o) => {
            let dest = account_id(&o.destination);
            out.trustline(&source, line(&source, &o.send_asset), at("sender trustline"));
            out.account(&dest, at("destination account"));
            out.trustline(&dest, line(&dest, &o.dest_asset), at("destination trustline"));
            out.concerns.push(Concern::PathHops { path: at("payment path") });
        }
        OperationBody::ManageSellOffer(o) => {
            out.trustline(&source, line(&source, &o.selling), at("selling trustline"));
            out.trustline(&source, line(&source, &o.buying), at("buying trustline"));
            out.concerns.push(Concern::Dex { path: at("offer matching") });
        }
        OperationBody::ManageBuyOffer(o) => {
            out.trustline(&source, line(&source, &o.selling), at("selling trustline"));
            out.trustline(&source, line(&source, &o.buying), at("buying trustline"));
            out.concerns.push(Concern::Dex { path: at("offer matching") });
        }
        OperationBody::CreatePassiveSellOffer(o) => {
            out.trustline(&source, line(&source, &o.selling), at("selling trustline"));
            out.trustline(&source, line(&source, &o.buying), at("buying trustline"));
            out.concerns.push(Concern::Dex { path: at("offer matching") });
        }
        OperationBody::ChangeTrust(o) => {
            out.trustline(&source, change_trust_line(&source, &o.line), at("trustline"));
        }
        OperationBody::AllowTrust(o) => out.account(&o.trustor, at("trustor account")),
        OperationBody::SetTrustLineFlags(o) => {
            out.account(&o.trustor, at("trustor account"));
            out.trustline(&o.trustor, line(&o.trustor, &o.asset), at("trustor trustline"));
        }
        OperationBody::Clawback(o) => {
            let from = account_id(&o.from);
            out.account(&from, at("clawback source account"));
            out.trustline(&from, line(&from, &o.asset), at("clawback trustline"));
        }
        OperationBody::CreateClaimableBalance(o) => {
            out.trustline(&source, line(&source, &o.asset), at("source trustline"));
        }
        OperationBody::AccountMerge(dest) => {
            out.account(&account_id(dest), at("merge destination account"));
        }
        OperationBody::SetOptions(_)
        | OperationBody::ManageData(_)
        | OperationBody::BumpSequence(_)
        | OperationBody::Inflation => {}
        OperationBody::InvokeHostFunction(_)
        | OperationBody::ExtendFootprintTtl(_)
        | OperationBody::RestoreFootprint(_) => {}
        _ => out.concerns.push(Concern::Unsupported {
            path: at("operation"),
            reason: format!(
                "QuorumScope does not analyze which ledger keys a {name} operation touches. Only its source account was checked."
            ),
        }),
    }
}
