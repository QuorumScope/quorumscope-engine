use crate::client::RpcClient;
use crate::config::RpcConfig;
use crate::get_ledger_entries::GetLedgerEntriesRequest;
use crate::health::GetHealthRequest;
use stellar_xdr::{Limits, WriteXdr};

pub async fn run_live_verification() -> anyhow::Result<()> {
    println!("Starting LIVE RPC VERIFICATION against Soroban Testnet...");

    let config = RpcConfig::new("https://soroban-testnet.stellar.org:443")?;
    let client = RpcClient::new(config)?;

    let health_req = GetHealthRequest::new(1);
    let health_res: crate::health::GetHealthResponse = client.send_request(&health_req).await?;

    if let Some(err) = health_res.error {
        anyhow::bail!("Health RPC Error: {:?}", err);
    }

    let result = health_res.result.unwrap();
    println!("VERIFIED LIVE: Connected to Soroban Testnet");
    println!("VERIFIED LIVE: Latest Ledger: {}", result.latest_ledger);

    let key = quorumscope_xdr::config::frozen_ledger_keys_key();
    let key_b64 = key.to_xdr_base64(Limits::none())?;

    println!(
        "VERIFIED LIVE: Querying FrozenLedgerKeys config setting (Key: {})",
        key_b64
    );

    let req = GetLedgerEntriesRequest::new(2, vec![key_b64]);
    let res: crate::get_ledger_entries::GetLedgerEntriesResponse =
        client.send_request(&req).await?;

    if let Some(err) = res.error {
        anyhow::bail!("Ledger Entries RPC Error: {:?}", err);
    }

    let entries_res = res.result.unwrap();
    println!(
        "VERIFIED LIVE: Result returned latest_ledger={}",
        entries_res.latest_ledger
    );

    if entries_res.entries.is_empty() {
        println!("VERIFIED LIVE: Frozen set is empty or does not exist at this ledger.");
    } else {
        println!(
            "VERIFIED LIVE: Frozen set exists. Base64 XDR: {}",
            entries_res.entries[0].xdr
        );
    }

    Ok(())
}
