use super::*;
use quorumscope_domain::preflight::PreflightResult;
use stellar_xdr::{
    AccountId, AlphaNum4, Asset, AssetCode4, BumpSequenceOp, ExtensionPoint, LedgerFootprint,
    LedgerKeyAccount, LedgerKeyTrustLine, ManageSellOfferOp, MuxedAccount, Operation,
    OperationBody, PathPaymentStrictReceiveOp, PaymentOp, Preconditions, Price, PublicKey,
    SequenceNumber, SorobanResources, SorobanTransactionData, SorobanTransactionDataExt,
    Transaction, TransactionExt, TransactionV1Envelope, TrustLineAsset, Uint256, VecM,
};

const PASSPHRASE: &str = "Test SDF Network ; September 2015";

fn account(byte: u8) -> AccountId {
    AccountId(PublicKey::PublicKeyTypeEd25519(Uint256([byte; 32])))
}

fn muxed(byte: u8) -> MuxedAccount {
    MuxedAccount::Ed25519(Uint256([byte; 32]))
}

fn usd(issuer: u8) -> Asset {
    Asset::CreditAlphanum4(AlphaNum4 {
        asset_code: AssetCode4(*b"USD\0"),
        issuer: account(issuer),
    })
}

fn envelope(source: u8, ops: Vec<OperationBody>, ext: TransactionExt) -> TransactionEnvelope {
    let operations: Vec<Operation> = ops
        .into_iter()
        .map(|body| Operation {
            source_account: None,
            body,
        })
        .collect();
    TransactionEnvelope::Tx(TransactionV1Envelope {
        tx: Transaction {
            source_account: muxed(source),
            fee: 100,
            seq_num: SequenceNumber(1),
            cond: Preconditions::None,
            memo: stellar_xdr::Memo::None,
            operations: operations.try_into().unwrap(),
            ext,
        },
        signatures: VecM::default(),
    })
}

fn payment(to: u8, asset: Asset) -> OperationBody {
    OperationBody::Payment(PaymentOp {
        destination: muxed(to),
        asset,
        amount: 10,
    })
}

fn account_key_id(byte: u8) -> FrozenKeyId {
    key_id(&LedgerKey::Account(LedgerKeyAccount {
        account_id: account(byte),
    }))
}

fn set<T: std::hash::Hash + Eq>(items: impl IntoIterator<Item = T>) -> HashSet<T> {
    items.into_iter().collect()
}

fn status_of(analysis: &Analysis) -> PreflightStatus {
    PreflightResult::derive_status(&analysis.findings, analysis.bypassed)
}

#[test]
fn empty_freeze_set_is_clear_and_deterministic() {
    let env = envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &HashSet::new(), &HashSet::new());
    assert!(analysis.findings.is_empty());
    assert_eq!(status_of(&analysis), PreflightStatus::Clear);
    assert_eq!(
        PreflightResult::derive_confidence(&analysis.findings),
        PreflightConfidence::Deterministic
    );
}

#[test]
fn unrelated_frozen_key_does_not_block() {
    let env = envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(9)]), &HashSet::new());
    assert_eq!(status_of(&analysis), PreflightStatus::Clear);
}

#[test]
fn frozen_destination_account_blocks_with_the_key_and_path() {
    let env = envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(2)]), &HashSet::new());
    assert_eq!(status_of(&analysis), PreflightStatus::BlockedValidation);
    let finding = &analysis.findings[0];
    assert_eq!(finding.confidence, PreflightConfidence::Deterministic);
    assert_eq!(finding.implicated_keys, vec![account_key_id(2)]);
    assert_eq!(
        finding.protocol_path,
        "operation 0 (Payment): destination account"
    );
}

#[test]
fn frozen_transaction_source_blocks() {
    let env = envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(1)]), &HashSet::new());
    assert_eq!(status_of(&analysis), PreflightStatus::BlockedValidation);
    assert_eq!(
        analysis.findings[0].protocol_path,
        "transaction source account"
    );
}

#[test]
fn frozen_destination_trustline_blocks() {
    let trustline = key_id(&LedgerKey::Trustline(LedgerKeyTrustLine {
        account_id: account(2),
        asset: TrustLineAsset::CreditAlphanum4(AlphaNum4 {
            asset_code: AssetCode4(*b"USD\0"),
            issuer: account(7),
        }),
    }));
    let env = envelope(1, vec![payment(2, usd(7))], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &set([trustline]), &HashSet::new());
    assert_eq!(status_of(&analysis), PreflightStatus::BlockedValidation);
    assert_eq!(
        analysis.findings[0].protocol_path,
        "operation 0 (Payment): destination trustline"
    );
}

#[test]
fn issuer_holds_no_trustline_for_its_own_asset() {
    let env = envelope(1, vec![payment(7, usd(7))], TransactionExt::V0);
    let touched = touches::collect(&env);
    let holders: Vec<_> = touched
        .keys
        .iter()
        .filter_map(|t| match &t.key {
            LedgerKey::Trustline(line) => Some(line.account_id.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        holders,
        vec![account(1)],
        "only the sender holds a USD trustline"
    );
}

fn soroban_ext(read_only: Vec<LedgerKey>, read_write: Vec<LedgerKey>) -> TransactionExt {
    TransactionExt::V1(SorobanTransactionData {
        ext: SorobanTransactionDataExt::V0,
        resources: SorobanResources {
            footprint: LedgerFootprint {
                read_only: read_only.try_into().unwrap(),
                read_write: read_write.try_into().unwrap(),
            },
            instructions: 0,
            disk_read_bytes: 0,
            write_bytes: 0,
        },
        resource_fee: 0,
    })
}

#[test]
fn soroban_footprint_intersections_name_read_only_and_read_write() {
    let key = |byte| {
        LedgerKey::Account(LedgerKeyAccount {
            account_id: account(byte),
        })
    };
    let env = envelope(
        1,
        vec![OperationBody::RestoreFootprint(
            stellar_xdr::RestoreFootprintOp {
                ext: ExtensionPoint::V0,
            },
        )],
        soroban_ext(vec![key(4)], vec![key(5)]),
    );
    let analysis = analyze(
        &env,
        PASSPHRASE,
        &set([account_key_id(4), account_key_id(5)]),
        &HashSet::new(),
    );
    let paths: Vec<_> = analysis
        .findings
        .iter()
        .map(|f| f.protocol_path.as_str())
        .collect();
    assert_eq!(
        paths,
        [
            "Soroban footprint (read-only)",
            "Soroban footprint (read-write)"
        ]
    );
    assert_eq!(status_of(&analysis), PreflightStatus::BlockedValidation);
}

#[test]
fn active_bypass_for_the_content_hash_allows_but_keeps_findings() {
    let env = envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0);
    let hash = content_hash(&env, PASSPHRASE);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(2)]), &set([hash]));
    assert!(analysis.bypassed);
    assert_eq!(analysis.transaction_hash, hash.to_string());
    assert_eq!(status_of(&analysis), PreflightStatus::AllowedByBypass);
    assert_eq!(
        analysis.findings.len(),
        1,
        "the blocked finding stays visible"
    );
}

#[test]
fn content_hash_depends_on_network_and_transaction() {
    let a = envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0);
    let b = envelope(1, vec![payment(3, Asset::Native)], TransactionExt::V0);
    assert_ne!(content_hash(&a, PASSPHRASE), content_hash(&b, PASSPHRASE));
    assert_ne!(
        content_hash(&a, PASSPHRASE),
        content_hash(&a, "Public Global Stellar Network ; September 2015")
    );
    assert_eq!(content_hash(&a, PASSPHRASE), content_hash(&a, PASSPHRASE));
}

#[test]
fn offers_are_dex_conditional_only_when_a_freeze_set_exists() {
    let offer = OperationBody::ManageSellOffer(ManageSellOfferOp {
        selling: Asset::Native,
        buying: usd(7),
        amount: 10,
        price: Price { n: 1, d: 1 },
        offer_id: 0,
    });
    let env = envelope(1, vec![offer], TransactionExt::V0);
    let clear = analyze(&env, PASSPHRASE, &HashSet::new(), &HashSet::new());
    assert_eq!(status_of(&clear), PreflightStatus::Clear);
    let conditional = analyze(&env, PASSPHRASE, &set([account_key_id(9)]), &HashSet::new());
    assert_eq!(status_of(&conditional), PreflightStatus::DexConditional);
    assert_eq!(
        PreflightResult::derive_confidence(&conditional.findings),
        PreflightConfidence::Conditional
    );
}

#[test]
fn path_payments_are_dex_conditional_not_a_failure() {
    let op = OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
        send_asset: Asset::Native,
        send_max: 10,
        destination: muxed(2),
        dest_asset: Asset::Native,
        dest_amount: 5,
        path: VecM::default(),
    });
    let env = envelope(1, vec![op], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(9)]), &HashSet::new());
    assert_eq!(status_of(&analysis), PreflightStatus::DexConditional);
}

fn claim_balance_envelope() -> TransactionEnvelope {
    envelope(
        1,
        vec![OperationBody::ClaimClaimableBalance(
            stellar_xdr::ClaimClaimableBalanceOp {
                balance_id: stellar_xdr::ClaimableBalanceId::ClaimableBalanceIdTypeV0(
                    stellar_xdr::Hash([3; 32]),
                ),
            },
        )],
        TransactionExt::V0,
    )
}

#[test]
fn opaque_identifier_operations_carry_apply_time_risk() {
    let analysis = analyze(
        &claim_balance_envelope(),
        PASSPHRASE,
        &set([account_key_id(9)]),
        &HashSet::new(),
    );
    assert_eq!(status_of(&analysis), PreflightStatus::ApplyTimeRisk);
    assert_eq!(
        PreflightResult::derive_confidence(&analysis.findings),
        PreflightConfidence::Conditional
    );
    let none = analyze(
        &claim_balance_envelope(),
        PASSPHRASE,
        &HashSet::new(),
        &HashSet::new(),
    );
    assert_eq!(status_of(&none), PreflightStatus::Clear);
}

#[test]
fn bypass_does_not_cover_apply_time_behavior() {
    let env = claim_balance_envelope();
    let hash = content_hash(&env, PASSPHRASE);
    let frozen = set([account_key_id(1)]);
    let analysis = analyze(&env, PASSPHRASE, &frozen, &set([hash]));
    assert!(analysis.bypassed);
    assert!(
        analysis
            .findings
            .iter()
            .any(|f| f.status == PreflightStatus::BlockedValidation)
    );
    assert_eq!(status_of(&analysis), PreflightStatus::ApplyTimeRisk);
}

#[test]
fn bypass_for_a_transaction_that_touches_nothing_frozen_is_clear() {
    let env = envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0);
    let hash = content_hash(&env, PASSPHRASE);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(9)]), &set([hash]));
    assert!(analysis.bypassed);
    assert_eq!(status_of(&analysis), PreflightStatus::Clear);
}

#[test]
fn credit_payment_to_a_frozen_account_is_not_blocked_without_its_trustline() {
    let env = envelope(1, vec![payment(2, usd(7))], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(2)]), &HashSet::new());
    assert_eq!(status_of(&analysis), PreflightStatus::Clear);
}

#[test]
fn fee_bump_outer_hash_and_both_sources_are_checked() {
    let inner = match envelope(1, vec![payment(2, Asset::Native)], TransactionExt::V0) {
        TransactionEnvelope::Tx(v1) => v1,
        _ => unreachable!(),
    };
    let bump = TransactionEnvelope::TxFeeBump(stellar_xdr::FeeBumpTransactionEnvelope {
        tx: stellar_xdr::FeeBumpTransaction {
            fee_source: muxed(6),
            fee: 200,
            inner_tx: stellar_xdr::FeeBumpTransactionInnerTx::Tx(inner.clone()),
            ext: stellar_xdr::FeeBumpTransactionExt::V0,
        },
        signatures: VecM::default(),
    });
    let blocked = analyze(
        &bump,
        PASSPHRASE,
        &set([account_key_id(6)]),
        &HashSet::new(),
    );
    assert_eq!(blocked.findings[0].protocol_path, "fee source account");
    let inner_hash = content_hash(&TransactionEnvelope::Tx(inner), PASSPHRASE);
    let wrapped = analyze(
        &bump,
        PASSPHRASE,
        &set([account_key_id(6)]),
        &set([inner_hash]),
    );
    assert!(
        !wrapped.bypassed,
        "an inner hash must not bypass a fee bump"
    );
}

#[test]
fn blocked_outranks_dex_conditional() {
    let op = OperationBody::PathPaymentStrictReceive(PathPaymentStrictReceiveOp {
        send_asset: Asset::Native,
        send_max: 10,
        destination: muxed(2),
        dest_asset: Asset::Native,
        dest_amount: 5,
        path: VecM::default(),
    });
    let env = envelope(1, vec![op], TransactionExt::V0);
    let analysis = analyze(&env, PASSPHRASE, &set([account_key_id(2)]), &HashSet::new());
    assert_eq!(analysis.findings.len(), 2);
    assert_eq!(status_of(&analysis), PreflightStatus::BlockedValidation);
}

#[test]
fn unmodeled_operation_is_reported_as_unsupported_analysis() {
    let env = envelope(
        1,
        vec![OperationBody::EndSponsoringFutureReserves],
        TransactionExt::V0,
    );
    let analysis = analyze(&env, PASSPHRASE, &HashSet::new(), &HashSet::new());
    assert_eq!(status_of(&analysis), PreflightStatus::UnsupportedAnalysis);
    assert_eq!(
        PreflightResult::derive_confidence(&analysis.findings),
        PreflightConfidence::InsufficientInformation
    );
    assert!(
        analysis.findings[0]
            .explanation
            .contains("EndSponsoringFutureReserves")
    );
}

#[test]
fn operations_that_only_touch_their_source_are_modeled() {
    let env = envelope(
        1,
        vec![OperationBody::BumpSequence(BumpSequenceOp {
            bump_to: SequenceNumber(5),
        })],
        TransactionExt::V0,
    );
    let analysis = analyze(&env, PASSPHRASE, &HashSet::new(), &HashSet::new());
    assert!(analysis.findings.is_empty());
}
