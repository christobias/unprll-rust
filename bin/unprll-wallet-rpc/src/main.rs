use std::sync::{Arc, RwLock};

use log::{error, info};
use structopt::StructOpt;

mod api_definitions;
mod config;
mod rpc_server;
mod wallet_store;

pub use config::Config;
use wallet_store::WalletStore;

#[tokio::main]
async fn main() {
    let config = Config::from_args();

    let addr = format!("127.0.0.1:{}", config.rpc_bind_port)
        .parse()
        .unwrap();
    bin_common::logger::init(&config.bin_common_config, "unprll-wallet-rpc").unwrap();

    let wallet_store = Arc::from(RwLock::from(WalletStore::new(config)));

    info!("RPC server listening on {}", addr);

    hyper::Server::bind(&addr)
        .serve(
            rpc_server::build_server(wallet_store.clone())
                .map_err(|_| error!("Failed to start RPC server!"))
                .unwrap()
                .into_web_service(),
        )
        .await
        .unwrap();
}
