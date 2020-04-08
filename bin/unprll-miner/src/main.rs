use structopt::StructOpt;

mod config;
mod miner;
mod network;
mod state_machine;

use config::Config;
use state_machine::MinerStateMachine;

fn main() {
    let mut runtime = tokio::runtime::Builder::new()
        .threaded_scheduler()
        .thread_stack_size(4 * 1024 * 1024)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime!");

    let config = Config::from_args();

    bin_common::logger::init(&config.bin_common_config, "unprll-miner").unwrap();

    log::info!(
        "{}",
        format!(
            "{:?} - {:?}",
            coin_specific::COIN_NAME,
            coin_specific::VERSION
        )
    );

    match MinerStateMachine::new(&config) {
        Ok(miner_state_machine) => {
            let fut = miner_state_machine.into_future();
            if let Err(err) = runtime.block_on(fut) {
                log::error!("Miner enountered an error: {}", err)
            }
        }
        Err(err) => log::error!("Failed to start miner: {}", err),
    }
}
