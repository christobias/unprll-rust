use serde_json::Value;

use async_jsonrpc_client::{JSONRPCClient, Result};
use common::Block;
use rpc::api_definitions::*;

use crate::config::Config;

pub struct Network {
    client: JSONRPCClient,
}

impl Network {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            client: JSONRPCClient::new(&config.daemon_address)?,
        })
    }
    pub async fn get_stats(&self) -> Result<GetStatsResponse> {
        self.client
            .send_jsonrpc_request("get_stats", Value::Null)
            .await
            .map(|stats| stats.unwrap())
    }
    pub async fn submit_block(&self, block: Block) -> Result<()> {
        self.client
            .send_jsonrpc_request::<()>(
                "submit_block",
                Value::Array(vec![Value::String(hex::encode(
                    bincode::serialize(&block).unwrap(),
                ))]),
            )
            .await
            .map(|_| ())
    }
}
