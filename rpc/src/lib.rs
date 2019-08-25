#[macro_use] extern crate log;

use std::sync::{
    Arc,
    RwLock
};

use futures::future::Future;
use tokio::{
    runtime::Runtime
};

use common::{
    Config,
};
use cryptonote_core::CryptonoteCore;

pub mod api_definitions;
mod rpc_server;

pub fn init(config: &Config, runtime: &mut Runtime, core: Arc<RwLock<CryptonoteCore>>) {
    let addr = format!("127.0.0.1:{}", config.rpc_bind_port).parse().unwrap();

    let server = hyper::Server::bind(&addr)
        .serve(rpc_server::build_server(core).map_err(|_| error!("Failed to start RPC server!")).unwrap().into_web_service())
        .map_err(|e| error!("server error: {}", e));

    runtime.spawn(server);

    info!("RPC server listening on {}", addr);
}
