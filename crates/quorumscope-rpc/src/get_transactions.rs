use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub limit: u32,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionsRequestParams {
    #[serde(rename = "startLedger")]
    pub start_ledger: u32,
    pub pagination: Option<PaginationParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionsRequest {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
    pub params: GetTransactionsRequestParams,
}

impl GetTransactionsRequest {
    pub fn new(id: u32, start_ledger: u32, cursor: Option<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: "getTransactions".to_string(),
            params: GetTransactionsRequestParams {
                start_ledger,
                pagination: Some(PaginationParams { limit: 100, cursor }),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResultItem {
    pub status: String,
    #[serde(rename = "txHash")]
    pub tx_hash: String,
    pub envelope_xdr: String,
    pub result_xdr: String,
    pub result_meta_xdr: String,
    pub ledger: u32,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionsResult {
    pub transactions: Vec<TransactionResultItem>,
    pub latest_ledger: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTransactionsResponse {
    pub jsonrpc: String,
    pub id: u32,
    pub result: Option<GetTransactionsResult>,
    pub error: Option<crate::error::JsonRpcError>,
}
