#[macro_use] extern crate log;

use std::sync::RwLock;

use futures::future::Future;
use structopt::StructOpt;
use tokio::runtime::Runtime;

use common::Config;
use cryptonote_core::CryptonoteCore;
use p2p::P2P;

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
    let core = RwLock::new(CryptonoteCore::new(&config));

    let p2p = P2P::new(&config, core);
    p2p.init_server(&mut runtime)?;

    runtime.shutdown_on_idle().wait().unwrap();
    Ok(())
}
