use futures::future::Future;
use log::{
    info
};
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

    info!("{}", format!("{:?} - {:?}", coin_specific::coin_info::COIN_NAME, coin_specific::coin_info::VERSION));

    runtime.spawn(MinerStateMachine::new(&config));
    runtime.shutdown_on_idle().wait().unwrap();
}
