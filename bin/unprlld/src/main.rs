use std::sync::{Arc, RwLock};

use structopt::StructOpt;

use cryptonote_core::CryptonoteCore;

mod config;
use config::Config;

#[tokio::main]
async fn main() {
    // Command Line Arguments
    let config = Config::from_args();

    // Logging
    bin_common::logger::init(&config.bin_common_config, "unprlld")
        .expect("Failed to initialise logger");

    // Main
    if let Err(error) = run(config).await {
        log::error!("Daemon encountered an error: {}", error);
    }
}

async fn run(config: Config) -> Result<(), anyhow::Error> {
    log::info!(
        "{}",
        format!(
            "{:?} - {:?}",
            coin_specific::COIN_NAME,
            coin_specific::VERSION
        )
    );

    // Cryptonote Core Hub
    let core = Arc::new(RwLock::new(CryptonoteCore::new(
        coin_specific::Unprll,
        &config.cryptonote_core_config,
    )));

    futures::join!(
        p2p::init(&config.p2p_config, core.clone())?,
        rpc::init(&config.rpc_config, core)?,
    );

    log::info!("Exiting");
    Ok(())
}
