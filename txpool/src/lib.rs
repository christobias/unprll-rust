use std::sync::{
    Arc,
    RwLock
};

use failure::Error;

use blockchain::Blockchain;
use common::Transaction;
use crypto::Hash256;

type Result<T> = std::result::Result<T, Error>;

pub struct TXPool {
    blockchain: Arc<RwLock<Blockchain>>
}

impl TXPool {
    pub fn new(blockchain: Arc<RwLock<Blockchain>>) -> Self {
        TXPool {
            blockchain
        }
    }
    pub fn add_tx(&mut self, transaction: Transaction) -> Result<()> {
        unimplemented!()
    }
    pub fn take_tx(&self, txid: Hash256) -> Result<Transaction> {
        unimplemented!()
    }
}
