use std::collections::HashMap;
use std::fs::File;
use std::sync::{
    Arc,
    RwLock
};
use std::time::Duration;

use failure::{
    Error,
    format_err
};
use futures::{
    Async,
    future::{
        Future
    },
    Poll,
    stream::Stream,
    try_ready
};
use log::{
    debug,
    error
};
use tokio::timer::Interval;

use async_jsonrpc_client::{
    JSONRPCClient,
    serde_json
};
use coin_specific::{
    Unprll
};
use common::{
    GetHash,
    Transaction
};
use rpc::api_definitions::{
    GetBlocksRequest,
    GetBlocksResponse
};
use wallet::Wallet;

use crate::config::Config;

pub struct WalletStore {
    client: JSONRPCClient,
    refresh_interval: Interval,
    wallet_dir: std::path::PathBuf,
    wallets: HashMap<String, Arc<RwLock<Wallet<Unprll>>>>
}

impl WalletStore {
    pub fn new(config: Config) -> Self {
        let ws = WalletStore {
            client: JSONRPCClient::new(&config.daemon_address).unwrap(),
            refresh_interval: Interval::new_interval(Duration::from_secs(10)),
            wallet_dir: config.wallet_dir,
            wallets: HashMap::new()
        };

        std::fs::create_dir_all(&ws.wallet_dir).unwrap();

        ws
    }

    pub fn add_wallet(&mut self, wallet_name: String, wallet: Wallet<Unprll>) -> Result<(), Error> {
        if self.wallets.contains_key(&wallet_name) {
            return Err(format_err!("Wallet {} exists in memory", wallet_name))
        }
        self.wallets.insert(wallet_name, Arc::from(RwLock::new(wallet)));
        Ok(())
    }

    pub fn load_wallet(&mut self, wallet_name: String) -> Result<(), Error> {
        let mut wallet_path = self.wallet_dir.clone();
        wallet_path.push(&wallet_name);

        let wallet_file = File::open(wallet_path)?;

        let wallet = bincode::deserialize_from(wallet_file)?;
        self.add_wallet(wallet_name, wallet)
    }

    pub fn save_wallets(&self) -> Result<(), Error> {
        for (wallet_name, wallet) in &self.wallets {
            let mut wallet_path = self.wallet_dir.clone();
            wallet_path.push(wallet_name);

            let file = if std::fs::metadata(&wallet_path).is_err() {
                // Create the file
                File::create(wallet_path)
            } else {
                File::open(wallet_path)
            }?;

            // TODO: Add file encryption before release
            bincode::serialize_into(file, &*wallet.read().unwrap())?;
        }
        Ok(())
    }

    pub fn get_wallet(&self, wallet_name: &str) -> Result<Arc<RwLock<Wallet<Unprll>>>, Error> {
        self.wallets.get(wallet_name).cloned().ok_or_else(|| format_err!("Wallet {} not found", wallet_name))
    }

    pub fn refresh_wallet(&self, wallet_name: &str) {
        if let Some(wallet) = self.wallets.get(wallet_name) {
            debug!("Refreshing {}", wallet_name);

            let last_checked_height = {
                let wallet = wallet.read().unwrap();
                let last_check = wallet.get_last_checked_block();
                *last_check.0
            };
            let wallet = wallet.clone();

            tokio::spawn(
                self.client.send_jsonrpc_request(
                    "get_blocks",
                    serde_json::to_value(GetBlocksRequest {
                        from: last_checked_height,
                        to: None
                    }).unwrap()
                ).map(|response| {
                    let response: GetBlocksResponse = response.unwrap();
                    // TODO: Move this to a #[serde(with)] method
                    (
                        response.blocks
                            .into_iter()
                            .flat_map(hex::decode)
                            .flat_map(|block_blob| bincode_epee::deserialize(&block_blob))
                            .collect(),
                        response.transactions
                            .into_iter()
                            .flat_map(hex::decode)
                            .flat_map(|tx_blob| bincode_epee::deserialize(&tx_blob))
                            .map(|tx: Transaction| (tx.get_hash(), tx))
                            .collect()
                    )
                }).map(move |(blocks, transactions): (Vec<_>, HashMap<_, _>)| {
                    blocks.iter().for_each(|block| {
                        wallet.write().unwrap().scan_block(block, &transactions);
                    });
                }).map_err(|error| {
                    error!("Failed to refresh wallet: {}", error);
                })
            );
        }
    }
}

impl Future for WalletStore {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        try_ready!(self.refresh_interval.poll().map_err(|_| {}));
        self.wallets.keys().for_each(|wallet_name| self.refresh_wallet(&wallet_name));

        futures::task::current().notify();
        Ok(Async::NotReady)
    }
}
