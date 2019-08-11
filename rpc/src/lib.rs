#[macro_use] extern crate log;

use std::sync::Arc;

use futures::future::Future;
use hyper::{
    Server
};
use jsonrpc_core::IoHandler;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::{
    ServerBuilder,
    ServerHandler
};
use tokio::{
    runtime::Runtime
};

use common::Config;
use cryptonote_core::CryptonoteCore;

mod api_definitions;
use api_definitions::*;

pub fn init(config: &Config, runtime: &mut Runtime, core: CryptonoteCore) {
    let addr = format!("127.0.0.1:{}", config.rpc_bind_port).parse().unwrap();

    let mut io = IoHandler::new();
    let rpc_server = RPCServer {
        core
    };
    io.extend_with(rpc_server.to_delegate());

    let builder = Arc::from(ServerBuilder::new(io));

    // Hook up the JSONRPC I/O Handler to the hyper server
    let server = Server::bind(&addr).serve(move || -> Result<ServerHandler> {
        Ok(builder.get_handler())
    }).map_err(|_| {});

    runtime.spawn(server);

    info!("RPC server listening on {}", addr);
}

use jsonrpc_core::Result;

#[rpc]
pub trait RPC {
    #[rpc(name = "get_stats")]
    fn get_stats(&self) -> Result<Stats>;
}

pub struct RPCServer {
    core: CryptonoteCore
}

impl RPC for RPCServer {
    fn get_stats(&self) -> Result<Stats> {
        Ok(Stats {
            height: self.core.blockchain().read().unwrap().get_tail().0,
            target_height: 99999,
            difficulty: 0,
            net_type: "main".to_string(),
            tx_pool_count: 1
        })
    }
}
