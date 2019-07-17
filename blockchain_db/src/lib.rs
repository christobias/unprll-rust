extern crate common;
extern crate crypto;

use crypto::Hash256;
use common::{Block, Transaction};

mod error;
mod mem;

pub use error::Error;
pub use mem::BlockchainMemDB;

pub trait BlockchainDB {
    // DB Operations
    fn open(self, path: &str, flags: u8) -> Result<(), Error>;
    fn is_open(self) -> bool;
    fn is_read_only(self) -> bool;
    fn close(self);
    fn sync(self);
    fn set_safe_sync_mode(self, state: bool);
    fn reset(&mut self);
    fn size(self) -> usize;
    fn fixup(self);

    // Batch Operations
    // fn start_batch() -> Result<(), Error>;
    // fn stop_batch() -> Result<(), Error>;

    // Block
    fn add_block(&mut self, block: Block, block_weight: usize, cumulative_difficulty: usize, coins_generated: u64, transactions: Vec<Transaction>) -> Result<(), Error>;
    fn get_block_by_height(&self, height: u64) -> Result<&Block, Error>;
    fn get_block_by_hash(&self, block_id: Hash256) -> Result<&Block, Error>;
    fn get_cumulative_difficulty(self) -> u64;
    // Zero index height, for consistency
    fn get_height(&self) -> u64;
    fn pop_block(&mut self) -> Result<Block, Error>;

    // Confirmed Transactions
    fn add_transaction(self);
    fn get_transaction(self);

    // Unconfirmed Transactions
    fn add_txpool_transaction(self);
    fn get_txpool_transaction(self);
    fn get_txpool_transaction_count(self);
    fn remove_txpool_transaction(self);

    // Key Image
    fn has_key_image(self);
}
