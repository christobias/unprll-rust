use std::collections::HashMap;

use failure::{
    Error,
    format_err
};

use wallet::Wallet;

pub struct WalletStore {
    wallets: HashMap<String, Wallet>
}

impl WalletStore {
    pub fn new() -> Self {
        WalletStore {
            wallets: HashMap::new()
        }
    }
    pub fn add_wallet(&mut self, wallet_name: String, wallet: Wallet) -> Result<(), Error> {
        if self.wallets.contains_key(&wallet_name) {
            return Err(format_err!("Wallet {} exists in memory", wallet_name))
        }
        self.wallets.insert(wallet_name, wallet);
        Ok(())
    }
}
