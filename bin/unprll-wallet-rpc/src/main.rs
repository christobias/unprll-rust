use std::sync::{Arc, RwLock};

use jsonrpsee::{raw::RawServer, transport::http::HttpTransportServer};
use structopt::StructOpt;

pub mod api_definitions;
mod config;
mod rpc_server;
mod wallet_store;

pub use config::Config;
use rpc_server::WalletRPCServer;
use wallet_store::WalletStore;

#[tokio::main]
async fn main() {
    let config = Config::from_args();

    let addr = format!("127.0.0.1:{}", config.rpc_bind_port)
        .parse()
        .unwrap();
    bin_common::logger::init(&config.bin_common_config, "unprll-wallet-rpc").unwrap();

    let transport_server = HttpTransportServer::bind(&addr).await.unwrap();
    let server = RawServer::new(transport_server);

    let wallet_rpc_server =
        WalletRPCServer::new(server, Arc::from(RwLock::from(WalletStore::new(config))));

    log::info!("RPC server listening on {}", addr);
    wallet_rpc_server.run().await;
}
