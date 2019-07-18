#[macro_use] extern crate log;

use std::sync::{Arc, RwLock};

use futures::future::Future;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use common::Config;
use cryptonote_core::CryptonoteCore;

fn main() {
    // Command Line Arguments
    let config = Config::from_args();

    // Logging
    bin_common::logger::init(&config).expect("Failed to initialise logger");

    // Main
    run(config).expect("Failed to run daemon!");
    info!("Exiting");
}

fn run(config: Config) -> Result<(), std::io::Error> {
    info!("{}", format!("Unprll {} - {}", cryptonote_config::VERSION, cryptonote_config::RELEASE_NAME));
    let mut runtime = Runtime::new().unwrap();

    // Cryptonote Core Hub
    let core = Arc::new(RwLock::new(CryptonoteCore::new(&config)));

    p2p::init(&config, &mut runtime, core.clone())?;

    runtime.shutdown_on_idle().wait().unwrap();
    Ok(())
}
