use std::sync::{
    Arc,
    RwLock
};

use futures::future::Future;
use log::{
    error,
    info
};
use structopt::StructOpt;

mod api_definitions;
mod config;
mod rpc_server;
mod wallet_store;

pub use config::Config;
use wallet_store::WalletStore;

fn main() {
    let config = Config::from_args();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();

    let addr = format!("127.0.0.1:{}", config.rpc_bind_port).parse().unwrap();
    bin_common::logger::init(&config.bin_common_config, "unprll-wallet-rpc").unwrap();

    let wallet_store = Arc::from(RwLock::from(WalletStore::new(config)));

    let server = hyper::Server::bind(&addr)
        .serve(
            rpc_server::build_server(wallet_store.clone())
                .map_err(|_| error!("Failed to start RPC server!"))
                .unwrap()
                .into_web_service()
        )
        .map_err(|e| error!("server error: {}", e));

    info!("RPC server listening on {}", addr);
    let wallet_store = wallet_store.clone();

    runtime.spawn(server);
    runtime.spawn(futures::future::poll_fn(move || {
        wallet_store.write().unwrap().poll()
    }));

    runtime.shutdown_on_idle().wait().unwrap();
}
