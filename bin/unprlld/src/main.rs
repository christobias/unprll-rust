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
use tokio::runtime::Runtime;

use cryptonote_core::CryptonoteCore;

mod config;
use config::Config;

fn main() {
    // Command Line Arguments
    let config = Config::from_args();

    // Logging
    bin_common::logger::init(&config.bin_common_config, "unprlld").expect("Failed to initialise logger");

    // Main
    run(config).unwrap_or_else(|err| error!("Unable to run daemon! {}", err));
    info!("Exiting");
}

fn run(config: Config) -> Result<(), std::io::Error> {
    info!("{}", format!("{:?} - {:?}", coin_specific::coin_info::COIN_NAME, coin_specific::coin_info::VERSION));
    let mut runtime = Runtime::new()?;

    // Cryptonote Core Hub
    let core = Arc::new(RwLock::new(CryptonoteCore::new(&config.cryptonote_core_config)));

    p2p::init(&config.p2p_config, &mut runtime, core.clone())?;
    rpc::init(&config.rpc_config, &mut runtime, core);

    runtime.shutdown_on_idle().wait().unwrap_or_else(|_| error!("Runtime shut down abruptly!"));
    Ok(())
}
