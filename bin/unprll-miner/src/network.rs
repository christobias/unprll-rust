use futures::Future;
use serde_json::{
    Value
};

use async_jsonrpc_client::{
    Error,
    JSONRPCClient,
    Result
};
use common::Block;
use rpc::api_definitions::*;

use crate::config::Config;

pub struct Network {
    client: JSONRPCClient
}

impl Network {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            client: JSONRPCClient::new(&config.daemon_address)?
        })
    }
    pub fn get_stats(&self) -> impl Future<Item = Stats, Error = Error> {
        self.client.send_jsonrpc_request("get_stats", &[]).map(|x| x.unwrap())
    }
    pub fn submit_block(&self, block: Block) -> impl Future<Item = (), Error = Error> {
        self.client.send_jsonrpc_request::<()>(
            "submit_block",
            &[Value::String(hex::encode(bincode::serialize(&block).unwrap()))]
        ).map(|_| ())
    }
}
