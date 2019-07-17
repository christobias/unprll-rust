use std::collections::HashMap;

use common::{Block, GetHash, Transaction};
use crypto::Hash256;

use crate::error::Error as DBError;
use crate::BlockchainDB;

pub struct BlockchainMemDB {
    blocks: HashMap<Hash256, Block>,
    block_heights: HashMap<u64, Hash256>,
    transactions: HashMap<Hash256, Transaction>
}

impl BlockchainMemDB {
    pub fn new() -> BlockchainMemDB {
        BlockchainMemDB {
            blocks: HashMap::new(),
            block_heights: HashMap::new(),
            transactions: HashMap::new()
        }
    }
}

impl BlockchainDB for BlockchainMemDB {
    fn open(self, filename: &str, flags: u8) -> Result<(), DBError> {
        Ok(())
    }
    fn is_open(self) -> bool {
        true
    }
    fn is_read_only(self) -> bool {
        false
    }
    fn close(self) {

    }
    fn sync(self) {

    }
    fn set_safe_sync_mode(self, _state: bool) {

    }
    fn reset(&mut self) {
        self.blocks.clear();
        self.block_heights.clear();
        self.transactions.clear();
    }
    fn size(self) -> usize {
        std::mem::size_of_val(&self.blocks) + std::mem::size_of_val(&self.transactions)
    }
    fn fixup(self) {

    }

    // Block
    fn add_block(&mut self, block: Block, block_weight: usize, cumulative_difficulty: usize, coins_generated: u64, transactions: Vec<Transaction>) -> Result<(), DBError> {
        let block_id = &block.get_hash();
        if self.blocks.contains_key(block_id) {
            return Err(DBError::Exists(format!("Block with ID {} exists", hex::encode(block_id)).to_string()));
        }

        for tx in transactions.iter() {
            let txid = &tx.get_hash();
            if self.transactions.contains_key(txid) {
                return Err(DBError::Exists(format!("Transaction with ID {} exists", hex::encode(txid)).to_string()));
            }
        }

        self.blocks.insert(*block_id, block);
        for tx in transactions {
            let txid = &tx.get_hash();
            self.transactions.insert(*txid, tx);
        }
        Ok(())
    }
    fn get_block_by_height(&self, height: u64) -> Result<&Block, DBError> {
        let block_id = self.block_heights.get(&height).ok_or(DBError::DoesNotExist(format!("Block at height {} does not exist", height)))?;
        self.get_block_by_hash(*block_id)
    }
    fn get_block_by_hash(&self, block_id: Hash256) -> Result<&Block, DBError> {
        self.blocks.get(&block_id).ok_or(DBError::DoesNotExist(format!("Block with ID {:?} does not exist", block_id)))
    }
    fn get_cumulative_difficulty(self) -> u64 {
        100
    }
    fn get_height(&self) -> u64 {
        (self.block_heights.len() - 1) as u64
    }
    fn pop_block(&mut self) -> Result<Block, DBError> {
        let height = self.get_height();
        let block_id = self.block_heights.get(&height).ok_or(DBError::DoesNotExist(format!("Block at height {} does not exist", height)))?;

        // At this point, it can be assumed the block exists on both tables
        let block = self.blocks.remove_entry(block_id).expect("Inconsistent state").1;
        Ok(block)
    }

    // Confirmed Transactions
    fn add_transaction(self) {

    }
    fn get_transaction(self) {

    }

    // Unconfirmed Transactions
    fn add_txpool_transaction(self) {

    }
    fn get_txpool_transaction(self) {

    }
    fn get_txpool_transaction_count(self) {

    }
    fn remove_txpool_transaction(self) {

    }

    fn has_key_image(self) {

    }
}
