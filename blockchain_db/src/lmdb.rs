extern crate lmdb;

use lmdb::Environment;
use std::error::Error;
use std::fs;
use std::path;

use crate::error::Error as DBError;
use crate::BlockchainDB;

pub struct BlockchainLMDB {
    env: Environment,
    open: bool
}

impl BlockchainDB for BlockchainLMDB {
    // Constructor
    fn new(path: &path::Path) -> Result<Box<BlockchainLMDB>, Error> {
        if !path.exists() {
            std::fs::create_dir(path)?;
        }
        let metadata = std::fs::metadata(path)?;
        if !metadata.is_dir() {
            Err(Error::IOError(""));
        }
        let db = BlockchainLMDB {
            env: Environment::new().open(std::path::Path::new(path))?
        };
        Ok(Box::new(db))
    }

    // DB Operations
    fn open() {

    }
    fn is_open() -> bool {
        open
    }
    fn is_read_only() {

    }
    fn close() {

    }
    fn sync() {

    }
    fn set_safe_sync_mode(state: bool) {

    }
    fn reset() {

    }
    fn size() {

    }
    fn fixup() {

    }

    // Batch Operations
    fn start_batch() -> Result<(), Error> {
        Ok(())
    }
    fn stop_batch() -> Result<(), Error> {
        Ok(())
    }

    // Block
    fn add_block() {

    }
    fn get_block_by_height() {

    }
    fn get_block_by_hash() {

    }
    fn get_cumulative_difficulty() {

    }
    fn get_height() {

    }
    fn pop_block() {

    }

    // Confirmed Transactions
    fn add_transaction() {

    }
    fn get_transaction() {

    }

    // Unconfirmed Transactions
    fn add_txpool_transaction() {

    }
    fn get_txpool_transaction() {

    }
    fn get_txpool_transaction_count() {

    }
    fn remove_txpool_transaction() {

    }

    // Key Image
    fn has_key_image() {

    }
}
