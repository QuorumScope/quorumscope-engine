use crate::error::JsonRpcError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLatestLedgerRequest {
    pub jsonrpc: String,
    pub id: u32,
    pub method: String,
}

impl GetLatestLedgerRequest {
    pub fn new(id: u32) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: "getLatestLedger".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLatestLedgerResult {
    pub id: String,
    #[serde(rename = "protocolVersion")]
    pub protocol_version: u32,
    pub sequence: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLatestLedgerResponse {
    pub jsonrpc: String,
    pub id: u32,
    pub result: Option<GetLatestLedgerResult>,
    pub error: Option<JsonRpcError>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_names_the_rpc_method() {
        let json = serde_json::to_string(&GetLatestLedgerRequest::new(7)).unwrap();
        assert!(json.contains("getLatestLedger"));
    }

    #[test]
    fn response_reads_protocol_version_and_sequence() {
        let body = r#"{"jsonrpc":"2.0","id":7,"result":{"id":"abc","protocolVersion":25,"sequence":1234}}"#;
        let res: GetLatestLedgerResponse = serde_json::from_str(body).unwrap();
        let result = res.result.unwrap();
        assert_eq!(result.protocol_version, 25);
        assert_eq!(result.sequence, 1234);
    }
}
