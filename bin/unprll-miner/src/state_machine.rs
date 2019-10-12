use std::convert::TryFrom;
use std::time::{
    Duration,
    Instant,
    SystemTime,
    UNIX_EPOCH
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
use coin_specific::Unprll;
use common::{
    Block,
    TXExtra,
    TXIn,
    TXOut,
    TXOutTarget
};
use crypto::{
    Hash256,
    KeyPair,
};
use rpc::api_definitions::*;
use wallet::address::Address;

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
    miner_address: Address<Unprll>,
    state: MinerState
}

use MinerState::*;

impl MinerStateMachine {
    pub fn new(config: &Config) -> Result<Self, failure::Error> {
        Ok(MinerStateMachine {
            client: Network::new(&config)?,
            check_interval: Interval::new(Instant::now(), Duration::from_secs(config.check_interval)),
            last_prev_id: None,
            miner: Miner::new(),
            miner_address: Address::try_from(config.miner_address.as_str())?,
            state: Idle
        })
    }

    fn construct_block_template(&self, current_height: u64, prev_id: Hash256) -> Block {
        let mut block = Block::default();

        // Header
        block.header.major_version = 9;
        block.header.minor_version = 9;
        block.header.timestamp = {
            let mut t = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            t = t % 600 + 300;
            t
        };
        block.header.prev_id = prev_id;
        block.header.miner_specific = self.miner_address.spend_public_key;

        // Miner transaction
        block.miner_tx.prefix.inputs.push(TXIn::Gen(current_height + 1));

        // HACK TODO FIXME: Make proper transaction output generation code. This
        //                  just exists to test output scanning
        {
            let random_scalar = KeyPair::generate().secret_key;
            let tx_pub_key = random_scalar * crypto::ecc::BASEPOINT;

            let tx_scalar = crypto::ecc::data_to_scalar(&(random_scalar * self.miner_address.view_public_key.decompress().unwrap()));
            let tx_dest_key = tx_scalar * crypto::ecc::BASEPOINT + self.miner_address.spend_public_key.decompress().unwrap();

            block.miner_tx.prefix.outputs.push(TXOut {
                amount: 0,
                target: TXOutTarget::ToKey {
                    key: tx_dest_key.compress()
                }
            });
            block.miner_tx.prefix.extra.push(TXExtra::TxPublicKey(tx_pub_key.compress()));
        }

        block
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

                        // Create a new block template and reset the miner
                        let (height, prev_id) = stats.tail;
                        self.miner.set_block(Some(self.construct_block_template(height, Hash256::try_from(prev_id.as_str()).unwrap())));
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
