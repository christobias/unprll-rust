#![deny(missing_docs)]

//! # Cryptonote RPC server
//!
//! Used by wallets to access the Cryptonote core

use std::{
    future::Future,
    sync::{Arc, RwLock},
};

use cryptonote_core::{CryptonoteCore, EmissionCurve};
use jsonrpsee::{raw::RawServer, transport::http::HttpTransportServer};

pub mod api_definitions;
mod config;
mod rpc_server;

pub use config::Config;
use rpc_server::DaemonRPCServer;

/// Initialize the RPC server
pub fn init<TCoin: 'static + EmissionCurve + Send + Sync>(
    config: &Config,
    core: Arc<RwLock<CryptonoteCore<TCoin>>>,
) -> Result<impl Future, failure::Error> {
    let addr = format!("127.0.0.1:{}", config.rpc_bind_port).parse()?;

    Ok(async move {
        let transport_server = HttpTransportServer::bind(&addr).await.unwrap();
        let server = RawServer::new(transport_server);

        let daemon_rpc_server = DaemonRPCServer::new(server, core);

        log::info!("RPC server listening on {}", addr);

        daemon_rpc_server.run().await;
    })
}
