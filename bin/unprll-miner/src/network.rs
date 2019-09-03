use futures::Future;
use reqwest::{
    r#async::Client,
    Error
};
use serde::{
    Deserialize,
    Serialize
};
use serde_json::{
    Value
};

use common::Block;
use rpc::api_definitions::*;

use crate::config::Config;

#[derive(Serialize)]
struct JSONRPCRequest {
    jsonrpc: String,
    method: String,
    params: Vec<Value>,
    id: u64
}

#[derive(Deserialize)]
struct JSONRPCResponse<T> {
    result: T
}

pub struct Network {
    client: Client,
    daemon_address: String
}

type Result<T> = std::result::Result<T, Error>;

impl Network {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Self {
            client: Client::builder().build()?,
            daemon_address: format!("http://{}", config.daemon_address)
        })
    }
    fn send_jsonrpc_request<T: for<'de> Deserialize<'de>>(&self, method: &str, params: &[Value]) -> impl Future<Item = JSONRPCResponse<T>, Error = Error> {
        self.client.post(&self.daemon_address)
            .json(&JSONRPCRequest {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params: params.to_vec(),
                id: 1
            })
            .send()
            .map_err(Error::from)
            .and_then(|mut res| {
                res.json()
            })
    }
    pub fn get_stats(&self) -> impl Future<Item = Stats, Error = Error> {
        self.send_jsonrpc_request("get_stats", &[])
            .map(|res| {
                res.result
            })
    }
    pub fn submit_block(&self, block: Block) -> impl Future<Item = (), Error = Error> {
        self.send_jsonrpc_request::<()>(
            "submit_block",
            &[Value::String(hex::encode(bincode::serialize(&block).unwrap()))]
        ).map(|_| {})
    }
}
