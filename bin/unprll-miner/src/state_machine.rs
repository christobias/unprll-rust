use std::convert::TryFrom;
use std::time::{
    Duration,
    Instant
};

use futures::{
    future::Future,
    prelude::*,
    stream::Stream,
    try_ready
};
use log::{
    error,
    info
};
use tokio::timer::Interval;

use async_jsonrpc_client::Error;
use crypto::Hash256;
use rpc::api_definitions::*;

use crate::config::Config;
use crate::miner::Miner;
use crate::network::Network;

enum MinerState {
    Idle,
    RequestingStats(Box<dyn Future<Item = GetStatsResponse, Error = Error> + Send>),
    Mining,
    SubmittingBlock(Box<dyn Future<Item = (), Error = Error> + Send>),
}

pub struct MinerStateMachine {
    client: Network,
    check_interval: Interval,
    last_prev_id: Option<String>,
    miner: Miner,
    state: MinerState
}

use MinerState::*;

impl MinerStateMachine {
    pub fn new(config: &Config) -> Self {
        MinerStateMachine {
            client: Network::new(&config).unwrap(),
            check_interval: Interval::new(Instant::now(), Duration::from_secs(config.check_interval)),
            last_prev_id: None,
            miner: Miner::new(),
            state: Idle
        }
    }
}

impl Future for MinerStateMachine {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        // Check if we need to check the daemon for a new chain tail
        if let Ok(Async::Ready(_)) = self.check_interval.poll() {
            info!("Checking for new chain head...");
            // Request the chain's current status
            self.state = RequestingStats(Box::new(self.client.get_stats()));
        }

        loop {
            match &mut self.state {
                Idle => {
                    return Ok(Async::NotReady);
                },
                RequestingStats(future) => {
                    let stats = try_ready!(future.map_err(|e| error!("{}", e)).poll());

                    // Check if the tail changed
                    let last_prev_id = self.last_prev_id.take();
                    let mut reset = false;
                    if let Some(last_prev_id) = last_prev_id {
                        if last_prev_id != stats.tail.1 {
                            // Tail has changed, reset
                            reset = true;
                        }
                        self.last_prev_id = Some(last_prev_id);
                    } else {
                        // Fresh start, reset anyway
                        info!("Starting miner...");
                        self.last_prev_id = last_prev_id;
                        reset = true;
                    }

                    if reset {
                        info!("New block was added to the chain. Resetting miner...");

                        // Create a new block template
                        let mut b = common::Block::genesis();
                        let (height, prev_id) = stats.tail;
                        b.miner_tx.prefix.inputs[0] = common::TXIn::Gen { height: height + 1 };
                        b.header.prev_id = Hash256::try_from(prev_id.as_str()).unwrap();

                        // Reset the miner
                        self.miner.set_block(Some(b));
                        self.miner.set_difficulty(stats.difficulty);

                        // Update our last seen tail
                        self.last_prev_id = Some(prev_id);
                    }

                    self.state = Mining;
                },
                Mining => {
                    let block = try_ready!(self.miner.poll());

                    info!("Block found!");
                    self.state = SubmittingBlock(Box::new(self.client.submit_block(block)))
                },
                SubmittingBlock(future) => {
                    try_ready!(future.map_err(|e| error!("{}", e)).poll());

                    self.state = RequestingStats(Box::new(self.client.get_stats()));
                }
            }
        }
    }
}
