#![deny(missing_docs)]

//! # Cryptonote RPC server
//!
//! Used by wallets to access the Cryptonote core

use std::{
    future::Future,
    sync::{Arc, RwLock},
};

use cryptonote_core::{CryptonoteCore, EmissionCurve};

pub mod api_definitions;
mod config;
mod rpc_server;

pub use config::Config;

/// Initialize the RPC server
pub fn init<TCoin: 'static + EmissionCurve + Send + Sync>(
    config: &Config,
    core: Arc<RwLock<CryptonoteCore<TCoin>>>,
) -> Result<impl Future, failure::Error> {
    let addr = format!("127.0.0.1:{}", config.rpc_bind_port)
        .parse()?;

    let server = hyper::Server::bind(&addr).serve(
        rpc_server::build_server(core)
            .map_err(|_| failure::format_err!("Failed to start RPC server!"))?
            .into_web_service(),
    );

    log::info!("RPC server listening on {}", addr);
    Ok(server)
}
