use std::{
    convert::TryFrom,
    future::Future,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use jsonrpsee::{raw::RawClient, transport::http::HttpTransportClient};

use coin_specific::{emission::EmissionCurve, Unprll};
use common::{Block, TXExtra, TXIn, TXOut, TXOutTarget};
use crypto::{Hash256, KeyPair};
use rpc::api_definitions::DaemonRPC;
use transaction_util::{address::Address, Derivation};

use crate::config::Config;
use crate::miner::Miner;

pub struct MinerStateMachine {
    check_interval: Duration,
    daemon_address: String,
    last_checked: Instant,
    last_prev_id: Option<String>,
    miner: Miner,
    miner_address: Address,
}

impl MinerStateMachine {
    pub fn new(config: Config) -> Result<Self, anyhow::Error> {
        Ok(MinerStateMachine {
            check_interval: Duration::from_secs(config.check_interval),
            daemon_address: config.daemon_address,
            last_checked: Instant::now(),
            last_prev_id: None,
            miner: Miner::new(),
            miner_address: Address::from_address_string::<Unprll>(config.miner_address.as_str())?,
        })
    }

    // TODO FIXME: jsonrpsee usese a background thread to maintain its requests which puts the CPU under
    //             constant load. Remove this once that's changed
    fn get_rpc_client(&self) -> RawClient<HttpTransportClient> {
        RawClient::new(HttpTransportClient::new(&format!(
            "http://{}",
            self.daemon_address
        )))
    }

    fn construct_block_template(&self, current_height: u64, prev_id: Hash256) -> Block {
        let mut block = Block::default();

        // Header
        block.header.major_version = 9;
        block.header.minor_version = 9;
        block.header.timestamp = {
            let mut t = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            t = t % 600 + 300;
            t
        };
        block.header.prev_id = prev_id;
        block.header.miner_specific = self.miner_address.spend_public_key;

        // Miner transaction
        block
            .miner_tx
            .prefix
            .inputs
            .push(TXIn::Gen(current_height + 1));

        // This code is similar to transaction_util::construct_tx but is
        // much simpler since there are fewer cases to deal with and the
        // RingCT signature is unnecessary

        // Generate a random secret key for this output
        let random_scalar = KeyPair::generate().secret_key;
        // Set the transaction public key
        let tx_pub_key = &random_scalar * &crypto::ecc::BASEPOINT_TABLE;

        // Generate the transaction derivation and hence the keypair
        let tx_target_keypair =
            Derivation::from(&random_scalar, &self.miner_address.view_public_key)
                .unwrap()
                .to_keypair(0, self.miner_address.spend_public_key);

        block.miner_tx.prefix.outputs.push(TXOut {
            amount: Unprll.get_block_reward(block.header.major_version),
            target: TXOutTarget::ToKey {
                key: tx_target_keypair.public_key,
            },
        });
        block
            .miner_tx
            .prefix
            .extra
            .push(TXExtra::TxPublicKey(tx_pub_key));

        block
    }

    pub fn into_future(mut self) -> impl Future<Output = Result<(), anyhow::Error>> {
        async move {
            loop {
                // Check if we need to check the daemon for a new chain tail
                let stats = DaemonRPC::get_stats(&mut self.get_rpc_client()).await?;
                self.last_checked = Instant::now();

                // Check if the tail changed
                let reset = if let Some(last_prev_id) = &self.last_prev_id {
                    // Tail has changed if not equal, reset
                    *last_prev_id != stats.tail.1
                } else {
                    // Fresh start, reset anyway
                    log::info!("Starting miner...");
                    true
                };

                if reset {
                    log::info!("New block was added to the chain. Resetting miner...");

                    // Create a new block template and reset the miner
                    let (height, prev_id) = stats.tail;
                    self.miner.set_block(Some(self.construct_block_template(
                        height,
                        Hash256::try_from(prev_id.as_str()).unwrap(),
                    )));
                    self.miner.set_difficulty(stats.difficulty.into());

                    // Update our last seen tail
                    self.last_prev_id = Some(prev_id);
                }

                while self.last_checked.elapsed() < self.check_interval {
                    if self.miner.run_pow_step() {
                        log::info!("Block found!");
                        DaemonRPC::submit_block(
                            &mut self.get_rpc_client(),
                            hex::encode(
                                bincode::serialize(&self.miner.take_block().unwrap()).unwrap(),
                            ),
                        )
                        .await?;
                        break;
                    }
                }
            }
        }
    }
}
