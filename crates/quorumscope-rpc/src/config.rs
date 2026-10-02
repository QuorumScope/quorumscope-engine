use std::time::Duration;
use reqwest::{Client, IntoUrl};
use crate::error::RpcError;

#[derive(Debug, Clone)]
pub struct RpcConfig {
    pub endpoint: url::Url,
    pub timeout: Duration,
    pub max_retries: u32,
    pub base_backoff: Duration,
}

impl RpcConfig {
    pub fn new<U: IntoUrl>(endpoint: U) -> Result<Self, RpcError> {
        Ok(Self {
            endpoint: endpoint.into_url()?,
            timeout: Duration::from_secs(30),
            max_retries: 5,
            base_backoff: Duration::from_millis(500),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_config() {
        let config = RpcConfig::new("http://localhost:8000").unwrap();
        assert_eq!(config.endpoint.as_str(), "http://localhost:8000/");
        assert_eq!(config.max_retries, 5);
    }
}
