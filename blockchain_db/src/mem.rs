use std::collections::HashMap;
use std::convert::TryInto;

use common::{Block, GetHash, Transaction};
use crypto::Hash256;

use crate::error::Result;
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
    fn is_read_only(&self) -> bool {
        false
    }
    fn sync(&self) {

    }
    fn set_safe_sync_mode(&self, _state: bool) {

    }
    fn reset(&mut self) {
        self.blocks.clear();
        self.block_heights.clear();
        self.transactions.clear();
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self).try_into().unwrap_or(u64::max_value())
    }
    fn fixup(&self) {

    }

    // Block
    fn add_block(&mut self, block: Block, _block_weight: usize, _cumulative_difficulty: usize, _coins_generated: u64, transactions: Vec<Transaction>) -> Result<()> {
        let block_id = &block.get_hash();
        if self.blocks.contains_key(block_id) {
            return Err(format_err!("Block with ID {} exists", block_id));
        }

        for tx in transactions.iter() {
            let txid = &tx.get_hash();
            if self.transactions.contains_key(txid) {
                return Err(format_err!("Transaction with ID {} exists", txid));
            }
        }

        self.blocks.insert(block_id.to_owned(), block);
        for tx in transactions {
            let txid = &tx.get_hash();
            self.transactions.insert(txid.to_owned(), tx);
        }
        Ok(())
    }
    fn get_block_by_height(&self, height: u64) -> Result<Block> {
        let block_id = self.block_heights.get(&height).ok_or(format_err!("Block at height {} does not exist", height))?;
        self.get_block_by_hash(block_id)
    }
    fn get_block_by_hash(&self, block_id: &Hash256) -> Result<Block> {
        self.blocks.get(&block_id).cloned().ok_or(format_err!("Block with ID {:?} does not exist", block_id))
    }
    fn get_cumulative_difficulty(&self) -> u64 {
        100
    }
    fn get_height(&self) -> u64 {
        (self.block_heights.len() - 1) as u64
    }
    fn pop_block(&mut self) -> Result<Block> {
        let height = self.get_height();
        let block_id = self.block_heights.get(&height).ok_or(format_err!("Block at height {} does not exist", height))?;

        // At this point, it can be assumed the block exists on both tables
        let block = self.blocks.remove_entry(block_id).expect("Inconsistent state").1;
        Ok(block)
    }

    // Confirmed Transactions
    fn add_transaction(&mut self) {

    }
    fn get_transaction(&self, id: &Hash256) -> Result<Transaction> {
        self.transactions.get(id).cloned().ok_or(format_err!("Transaction with ID {:?} does not exist", id))
    }

    // Unconfirmed Transactions
    fn add_txpool_transaction(&mut self) {

    }
    fn get_txpool_transaction(&self) {

    }
    fn get_txpool_transaction_count(&self) {

    }
    fn remove_txpool_transaction(&mut self) {

    }

    fn has_key_image(&self) {

    }
}
