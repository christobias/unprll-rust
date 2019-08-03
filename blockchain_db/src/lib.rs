#[macro_use] extern crate failure;

use crypto::Hash256;
use common::{Block, Transaction};

mod error;
mod lmdb;
mod mem;

pub use error::Result;
pub use lmdb::BlockchainLMDB;
pub use mem::BlockchainMemDB;

pub trait BlockchainDB {
    // DB Operations
    fn is_read_only(&self) -> bool;
    fn sync(&self);
    fn set_safe_sync_mode(&self, state: bool);
    fn reset(&mut self);
    fn size(&self) -> u64;
    fn fixup(&self);

    // Batch Operations
    // fn start_batch() -> Result<()>;
    // fn stop_batch() -> Result<()>;

    // Block
    fn add_block(&mut self, block: Block, block_weight: usize, cumulative_difficulty: usize, coins_generated: u64, transactions: Vec<Transaction>) -> Result<()>;
    fn get_block_by_height(&self, height: u64) -> Result<Block>;
    fn get_block_by_hash(&self, block_id: &Hash256) -> Result<Block>;
    fn get_cumulative_difficulty(&self) -> u64;
    // Zero index height, for consistency
    fn get_height(&self) -> u64;
    fn pop_block(&mut self) -> Result<Block>;

    // Confirmed Transactions
    fn add_transaction(&mut self);
    fn get_transaction(&self, id: &Hash256) -> Result<Transaction>;

    // Unconfirmed Transactions
    fn add_txpool_transaction(&mut self);
    fn get_txpool_transaction(&self);
    fn get_txpool_transaction_count(&self);
    fn remove_txpool_transaction(&mut self);

    // Key Image
    fn has_key_image(&self);
}
