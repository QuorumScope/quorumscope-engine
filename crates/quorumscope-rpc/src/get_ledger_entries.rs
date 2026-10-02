use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLedgerEntriesRequestParams {
    pub keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLedgerEntriesRequest {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
    pub params: GetLedgerEntriesRequestParams,
}

impl GetLedgerEntriesRequest {
    pub fn new(id: u32, keys: Vec<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: "getLedgerEntries".to_string(),
            params: GetLedgerEntriesRequestParams { keys },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntryResult {
    pub key: String,
    pub xdr: String,
    #[serde(rename = "lastModifiedLedgerSeq")]
    pub last_modified_ledger_seq: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLedgerEntriesResult {
    #[serde(rename = "latestLedger")]
    pub latest_ledger: u32,
    pub entries: Vec<LedgerEntryResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLedgerEntriesResponse {
    pub jsonrpc: String,
    pub id: u32,
    pub result: Option<GetLedgerEntriesResult>,
    pub error: Option<crate::error::JsonRpcError>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = GetLedgerEntriesRequest::new(1, vec!["key1".to_string()]);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("getLedgerEntries"));
        assert!(json.contains("key1"));
    }
}
