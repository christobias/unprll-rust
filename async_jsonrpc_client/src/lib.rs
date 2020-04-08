use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use serde_json;

#[derive(Serialize)]
struct JSONRPCRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: Value,
}

#[derive(Deserialize, Debug)]
struct JSONRPCError {
    code: i64,
    message: String,
}

impl std::fmt::Display for JSONRPCError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for JSONRPCError {}

// TODO: Look at other libraries
#[derive(Deserialize)]
pub struct JSONRPCResponse<T> {
    result: Option<T>,
    error: Option<JSONRPCError>,
}

pub struct JSONRPCClient {
    client: Client,
    address: String,
}

pub type Result<T> = std::result::Result<T, failure::Error>;
pub type Error = failure::Error;

impl JSONRPCClient {
    pub fn new(address: &str) -> Result<Self> {
        Ok(Self {
            client: Client::new(),
            address: format!("http://{}", address),
        })
    }
    pub async fn send_jsonrpc_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<Option<T>> {
        let res: JSONRPCResponse<T> = self
            .client
            .post(&self.address)
            .json(&JSONRPCRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params,
                id: 1,
            })
            .send()
            .await?
            .json()
            .await?;

        if let Some(err) = res.error {
            return Err(err.into());
        }

        if let Some(res) = res.result {
            Ok(Some(res))
        } else {
            Ok(None)
        }
    }
}
