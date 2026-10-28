use base64::{Engine as _, engine::general_purpose::STANDARD};
use quorumscope_preflight::content_hash;
use serde::Serialize;
use stellar_xdr::{
    AccountId, Asset, ConfigSettingEntry, EncodedLedgerKey, FreezeBypassTxs, FrozenLedgerKeys,
    Hash, LedgerEntryData, LedgerKey, LedgerKeyAccount, Limits, Memo, MuxedAccount, Operation,
    OperationBody, PaymentOp, Preconditions, PublicKey, SequenceNumber, Transaction,
    TransactionEnvelope, TransactionExt, TransactionV1Envelope, Uint256, VecM, WriteXdr,
};

const PASSPHRASE: &str = "Test SDF Network ; September 2015";

#[derive(Serialize)]
struct Fixture {
    protocol_version: u32,
    latest_ledger: u32,
    frozen_account: String,
    frozen_ledger_keys_xdr: String,
    freeze_bypass_txs_xdr: String,
    blocked_transaction_xdr: String,
    bypassed_transaction_xdr: String,
    bypass_hash: String,
}

fn account(byte: u8) -> AccountId {
    AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([byte; 32])))
}

fn transaction(sequence: u64, memo: Memo) -> TransactionEnvelope {
    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: Transaction {
            source_account: MuxedAccount::Ed25519(Uint256([1; 32])),
            fee: 100,
            seq_num: SequenceNumber(sequence.try_into().expect("fixture sequence fits i64")),
            cond: Preconditions::None,
            memo,
            operations: VecM::try_from(vec![Operation {
                source_account: None,
                body: OperationBody::Payment(PaymentOp {
                    destination: MuxedAccount::Ed25519(Uint256([5; 32])),
                    asset: Asset::Native,
                    amount: 100,
                }),
            }])
            .expect("fixture operation fits XDR bounds"),
            ext: TransactionExt::V0,
        },
        signatures: VecM::default(),
    })
}

fn entry_xdr(setting: ConfigSettingEntry) -> String {
    let data = LedgerEntryData::ConfigSetting(setting)
        .to_xdr(Limits::none())
        .expect("fixture config setting encodes");
    STANDARD.encode(data)
}

fn envelope_xdr(envelope: &TransactionEnvelope) -> String {
    STANDARD.encode(
        envelope
            .to_xdr(Limits::none())
            .expect("fixture envelope encodes"),
    )
}

fn main() {
    let frozen_key = LedgerKey::Account(LedgerKeyAccount {
        account_id: account(5),
    });
    let frozen_key_xdr = frozen_key
        .to_xdr(Limits::none())
        .expect("fixture key encodes");
    let frozen = entry_xdr(ConfigSettingEntry::FrozenLedgerKeys(FrozenLedgerKeys {
        keys: VecM::try_from(vec![EncodedLedgerKey(
            frozen_key_xdr
                .try_into()
                .expect("ledger key fits EncodedLedgerKey"),
        )])
        .expect("fixture key list fits XDR bounds"),
    }));

    let blocked = transaction(1, Memo::None);
    let bypassed = transaction(
        2,
        Memo::Text(stellar_xdr::String32::try_from("bypassed").expect("fixture memo fits")),
    );
    let bypass_hash = content_hash(&bypassed, PASSPHRASE);
    let bypass = entry_xdr(ConfigSettingEntry::FreezeBypassTxs(FreezeBypassTxs {
        tx_hashes: VecM::try_from(vec![Hash(*bypass_hash.as_bytes())])
            .expect("fixture bypass list fits XDR bounds"),
    }));

    let fixture = Fixture {
        protocol_version: 29,
        latest_ledger: 900,
        frozen_account: account(5).to_string(),
        frozen_ledger_keys_xdr: frozen,
        freeze_bypass_txs_xdr: bypass,
        blocked_transaction_xdr: envelope_xdr(&blocked),
        bypassed_transaction_xdr: envelope_xdr(&bypassed),
        bypass_hash: bypass_hash.to_string(),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&fixture).expect("fixture JSON serializes")
    );
}
