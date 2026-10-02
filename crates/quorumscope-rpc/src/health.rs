use crate::error::JsonRpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHealthRequest {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
}

impl GetHealthRequest {
    pub fn new(id: u32) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: "getHealth".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHealthResult {
    pub status: String,
    #[serde(rename = "latestLedger")]
    pub latest_ledger: u32,
    #[serde(rename = "latestLedgerCloseTime")]
    pub latest_ledger_close_time: String,
    #[serde(rename = "oldestLedger")]
    pub oldest_ledger: u32,
    #[serde(rename = "oldestLedgerCloseTime")]
    pub oldest_ledger_close_time: String,
    #[serde(rename = "ledgerRetentionWindow")]
    pub ledger_retention_window: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHealthResponse {
    pub jsonrpc: String,
    pub id: u32,
    pub result: Option<GetHealthResult>,
    pub error: Option<JsonRpcError>,
}
