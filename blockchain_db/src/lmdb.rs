use std::path;
use std::sync::{
    Arc,
    RwLock
};

use rkv::{
    Manager,
    Rkv
};

use common::{
    Block,
    Transaction
};
use crypto::Hash256;
use crate::{
    BlockchainDB,
    error::Result
};

pub struct BlockchainLMDB {
    env: Arc<RwLock<Rkv>>,
    data_dir: Box<path::Path>
}

impl BlockchainLMDB {
    // Constructor
    pub fn new(path: &path::Path) -> Result<BlockchainLMDB> {
        if !path.exists() {
            std::fs::create_dir(path)?;
        }
        let metadata = std::fs::metadata(path)?;
        if !metadata.is_dir() {
            return Err(format_err!("{} is not a directory", path.to_string_lossy()))
        }

        let manager_arc = Manager::singleton().write().unwrap().get_or_create(path, Rkv::new)?;

        let db = BlockchainLMDB {
            env: manager_arc,
            data_dir: Box::from(path)
        };
        Ok(db)
    }
}

impl BlockchainDB for BlockchainLMDB {
    // DB Operations
    fn is_read_only(&self) -> bool {
        false
    }
    fn sync(&self) {

    }
    fn set_safe_sync_mode(&self, state: bool) {

    }
    fn reset(&mut self) {

    }
    fn size(&self) -> u64 {
        let mut path = self.data_dir.clone().into_path_buf();
        path.push("data");
        path.set_extension("mdb");
        std::fs::metadata(path.into_boxed_path()).expect("Could not get metadata for db").len()
    }
    fn fixup(&self) {

    }

    // Block
    fn add_block(&mut self, block: Block, block_weight: usize, cumulative_difficulty: usize, coins_generated: u64, transactions: Vec<Transaction>) -> Result<()> {
        unimplemented!()
    }
    fn get_block_by_height(&self, height: u64) -> Result<Block> {
        unimplemented!()
    }
    fn get_block_by_hash(&self, hash: &Hash256) -> Result<Block> {
        unimplemented!()
    }
    fn get_cumulative_difficulty(&self) -> u64 {
        unimplemented!()
    }
    fn get_height(&self) -> u64 {
        unimplemented!()
    }
    fn pop_block(&mut self) -> Result<Block> {
        unimplemented!()
    }

    // Confirmed Transactions
    fn add_transaction(&mut self) {
        unimplemented!()
    }
    fn get_transaction(&self, txid: &Hash256) -> Result<Transaction> {
        unimplemented!()
    }

    // Unconfirmed Transactions
    fn add_txpool_transaction(&mut self) {
        unimplemented!()
    }
    fn get_txpool_transaction(&self) {
        unimplemented!()
    }
    fn get_txpool_transaction_count(&self) {
        unimplemented!()
    }
    fn remove_txpool_transaction(&mut self) {
        unimplemented!()
    }

    // Key Image
    fn has_key_image(&self) {
        unimplemented!()
    }
}
