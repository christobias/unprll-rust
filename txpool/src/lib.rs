use failure::Error;

use common::Transaction;
use crypto::Hash256;

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct TXPool {
}

impl TXPool {
    pub fn new() -> Self {
        TXPool { }
    }
    pub fn add_tx(&mut self, _transaction: Transaction) -> Result<()> {
        unimplemented!()
    }
    pub fn take_tx(&self, _txid: Hash256) -> Result<Transaction> {
        unimplemented!()
    }
}
