use std::collections::HashMap;

use failure::Error;

use common::{GetHash, Transaction};
use crypto::Hash256;

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct TXPool {
    transactions: HashMap<Hash256, Transaction>,
}

impl TXPool {
    pub fn new() -> Self {
        TXPool {
            transactions: HashMap::new(),
        }
    }
    pub fn add_tx(&mut self, transaction: Transaction) -> Result<()> {
        self.transactions
            .insert(transaction.get_hash(), transaction);

        Ok(())
    }
    pub fn has_tx(&self, txid: &Hash256) -> bool {
        self.transactions.contains_key(txid)
    }
}
