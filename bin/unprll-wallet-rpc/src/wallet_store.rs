use std::collections::HashMap;

use failure::{
    Error,
    format_err
};

use coin_specific::Unprll;
use wallet::Wallet;

pub struct WalletStore {
    wallets: HashMap<String, Wallet<Unprll>>
}

impl WalletStore {
    pub fn new() -> Self {
        WalletStore {
            wallets: HashMap::new()
        }
    }
    pub fn add_wallet(&mut self, wallet_name: String, wallet: Wallet<Unprll>) -> Result<(), Error> {
        if self.wallets.contains_key(&wallet_name) {
            return Err(format_err!("Wallet {} exists in memory", wallet_name))
        }
        self.wallets.insert(wallet_name, wallet);
        Ok(())
    }
    pub fn get_wallet(&self, wallet_name: &str) -> Result<&Wallet<Unprll>, Error> {
        self.wallets.get(wallet_name).ok_or_else(|| format_err!("Wallet {} not found", wallet_name))
    }
}
