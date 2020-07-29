use std::collections::HashMap;
use std::fs::File;
use std::sync::{Arc, RwLock};

use anyhow::Error;
use jsonrpsee::{raw::RawClient, transport::http::HttpTransportClient};

use common::{GetHash, Transaction};
use ensure_macro::ensure;
use rpc::api_definitions::DaemonRPC;
use wallet::Wallet;

use crate::config::Config;

pub struct WalletStore {
    // refresh_interval: Interval,
    rpc_client: RawClient<HttpTransportClient>,
    wallet_dir: std::path::PathBuf,
    wallets: HashMap<String, Arc<RwLock<Wallet>>>,
}

impl WalletStore {
    pub fn new(config: Config) -> Self {
        let ws = WalletStore {
            // refresh_interval: Interval::new_interval(Duration::from_secs(10)),
            rpc_client: RawClient::new(HttpTransportClient::new(&format!(
                "http://{}",
                config.daemon_address
            ))),
            wallet_dir: config.wallet_dir,
            wallets: HashMap::new(),
        };

        std::fs::create_dir_all(&ws.wallet_dir).unwrap();

        ws
    }
    pub fn add_wallet(&mut self, wallet_name: String, wallet: Wallet) -> Result<(), Error> {
        ensure!(
            !self.wallets.contains_key(&wallet_name),
            anyhow::format_err!("Wallet {} exists in memory", wallet_name)
        );
        self.wallets
            .insert(wallet_name, Arc::from(RwLock::new(wallet)));
        Ok(())
    }

    pub fn load_wallet(&mut self, wallet_name: String) -> Result<(), Error> {
        let mut wallet_path = self.wallet_dir.clone();
        wallet_path.push(&wallet_name);

        let wallet_file = File::open(wallet_path)?;

        let wallet = bincode::deserialize_from(wallet_file)?;
        self.add_wallet(wallet_name, wallet)
    }

    pub fn get_wallet(&self, wallet_name: &str) -> Option<Arc<RwLock<Wallet>>> {
        self.wallets.get(wallet_name).cloned()
    }

    pub async fn save_wallets(&self) -> Result<(), Error> {
        for (wallet_name, wallet) in &self.wallets {
            let mut wallet_path = self.wallet_dir.clone();
            wallet_path.push(wallet_name);

            // Open the file
            let file = File::create(wallet_path)?;

            // TODO: Add file encryption before release
            bincode::serialize_into(file, &*wallet.read().unwrap())?;
        }
        Ok(())
    }

    pub async fn refresh_wallets(&mut self) -> Result<(), Error> {
        for (wallet_name, wallet) in &self.wallets {
            log::debug!("Refreshing {}", wallet_name);

            let last_checked_height = {
                let wallet = wallet.read().unwrap();
                let last_check = wallet.get_last_checked_block();
                *last_check.0
            };
            let wallet = wallet.clone();

            let response =
                DaemonRPC::get_blocks(&mut self.rpc_client, last_checked_height, None)
                    .await?;

            // TODO: Move this to a #[serde(with)] method
            let blocks: Vec<common::Block> = response
                .blocks
                .into_iter()
                .flat_map(hex::decode)
                .flat_map(|block_blob| bincode::deserialize(&block_blob))
                .collect();

            let transactions: HashMap<_, _> = response
                .transactions
                .into_iter()
                .flat_map(hex::decode)
                .flat_map(|tx_blob| bincode::deserialize(&tx_blob))
                .map(|tx: Transaction| (tx.get_hash(), tx))
                .collect();

            blocks.iter().for_each(|block| {
                wallet.write().unwrap().scan_block(block, &transactions);
            });
        }

        Ok(())
    }
}
