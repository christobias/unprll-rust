use futures::future::Future;
use log::{error, info};
use structopt::StructOpt;

mod config;
mod miner;
mod network;
mod state_machine;

use config::Config;
use state_machine::MinerStateMachine;

fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .stack_size(4 * 1024 * 1024)
        .build()
        .expect("Failed to create Tokio runtime!");

    let config = Config::from_args();

    bin_common::logger::init(&config.bin_common_config, "unprll-miner").unwrap();

    info!(
        "{}",
        format!(
            "{:?} - {:?}",
            coin_specific::COIN_NAME,
            coin_specific::VERSION
        )
    );

    match MinerStateMachine::new(&config) {
        Ok(miner_state_machine) => {
            runtime.spawn(miner_state_machine);

            runtime.shutdown_on_idle().wait().unwrap()
        }
        Err(err) => error!("Failed to start miner: {}", err),
    }
}
