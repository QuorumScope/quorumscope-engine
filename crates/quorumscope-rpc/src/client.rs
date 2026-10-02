use reqwest::Client;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;
use crate::config::RpcConfig;
use crate::error::RpcError;

#[derive(Debug, Clone)]
pub struct RpcClient {
    pub config: RpcConfig,
    pub http: Client,
}

impl RpcClient {
    pub fn new(config: RpcConfig) -> Result<Self, RpcError> {
        let http = Client::builder()
            .timeout(config.timeout)
            .build()?;
            
        Ok(Self { config, http })
    }

    pub async fn send_request<Req, Res>(&self, request: &Req) -> Result<Res, RpcError>
    where
        Req: Serialize,
        Res: DeserializeOwned,
    {
        let endpoint = self.config.endpoint.clone();
        let http = self.http.clone();
        let req_json = serde_json::to_string(request)?;

        retry_loop(&self.config, || async {
            let res = http.post(endpoint.clone())
                .header("Content-Type", "application/json")
                .body(req_json.clone())
                .send()
                .await?;
            
            let status = res.status();
            if status.is_server_error() {
                // Return an error to trigger a retry
                return Err(RpcError::Http(res.error_for_status().unwrap_err()));
            }

            parse_json::<Res>(res).await
        }).await
    }
}

pub async fn retry_loop<T, F, Fut>(
    config: &RpcConfig,
    mut action: F,
) -> Result<T, RpcError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, RpcError>>,
{
    let mut attempt = 0;
    loop {
        match action().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                attempt += 1;
                if attempt > config.max_retries {
                    return Err(RpcError::MaxRetriesExceeded);
                }
                
                // Only retry on specific errors, but for simplicity we retry all in this loop
                // except maybe JSON parse errors?
                if let RpcError::Json(_) = e {
                    return Err(e);
                }

                let backoff = config.base_backoff * (2_u32.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }
        }
    }
}

pub async fn parse_json<T: DeserializeOwned>(response: reqwest::Response) -> Result<T, RpcError> {
    response.json::<T>().await.map_err(RpcError::Http)
}
