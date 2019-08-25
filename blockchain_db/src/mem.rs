use std::collections::HashMap;
use std::convert::TryInto;

use common::{
    Block,
    GetHash,
    Transaction
};
use crypto::{
    Hash256,
    KeyImage
};

use crate::error::Result;
use crate::BlockchainDBDriver;

pub struct BlockchainMemDB {
    blocks: HashMap<Hash256, Block>,
    block_heights: HashMap<u64, Hash256>,
    key_images: Vec<KeyImage>,
    transactions: HashMap<Hash256, Transaction>,
    unconfirmed_transactions: HashMap<Hash256, Transaction>
}

impl BlockchainMemDB {
    pub fn new() -> BlockchainMemDB {
        BlockchainMemDB {
            blocks: HashMap::new(),
            block_heights: HashMap::new(),
            key_images: Vec::new(),
            transactions: HashMap::new(),
            unconfirmed_transactions: HashMap::new()
        }
    }
}

impl BlockchainDBDriver for BlockchainMemDB {
    fn is_read_only(&self) -> bool {
        false
    }
    fn sync(&self) -> Result<()> {
        Ok(())
    }
    fn set_safe_sync_mode(&self, _state: bool) {

    }
    fn reset(&mut self) {
        self.blocks.clear();
        self.block_heights.clear();
        self.key_images.clear();
        self.transactions.clear();
        self.unconfirmed_transactions.clear();
    }
    fn size(&self) -> u64 {
        std::mem::size_of_val(&self).try_into().unwrap_or(u64::max_value())
    }
    fn fixup(&self) {

    }

    // Block
    fn add_block(&mut self, block: Block) -> Result<()> {
        let block_id = block.get_hash();
        self.blocks.insert(block_id.clone(), block);
        self.block_heights.insert(self.get_tail().map(|(current_height, _)| current_height + 1).unwrap_or(0), block_id);
        Ok(())
    }
    fn get_block_by_height(&self, height: u64) -> Option<Block> {
        let block_id = self.block_heights.get(&height)?;
        self.get_block_by_hash(block_id)
    }
    fn get_block_by_hash(&self, block_id: &Hash256) -> Option<Block> {
        self.blocks.get(&block_id).cloned()
    }
    fn get_cumulative_difficulty(&self) -> u64 {
        // TODO:
        100
    }
    fn get_tail(&self) -> Result<(u64, Block)> {
        let mut height: u64 = self.block_heights.iter().count().try_into().unwrap();
        if height != 0 { height -= 1 }

        Ok((height, self.get_block_by_height(height).ok_or_else(|| format_err!("Chain does not have any blocks"))?))
    }
    fn pop_block(&mut self) -> Result<Block> {
        let (height, _) = self.get_tail()?;
        let block_id = self.block_heights.get(&height).ok_or_else(|| format_err!("Block at height {} does not exist", height))?;

        // At this point, it can be assumed the block exists on both tables
        let (_, block) = self.blocks.remove_entry(block_id).expect("Inconsistent state");
        Ok(block)
    }

    fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        self.transactions.insert(transaction.get_hash(), transaction);
        Ok(())
    }
    fn get_transaction(&self, id: &Hash256) -> Option<Transaction> {
        self.transactions.get(id).cloned()
    }

    fn add_key_image(&mut self, key_image: KeyImage) -> Result<()> {
        self.key_images.push(key_image);
        Ok(())
    }
    fn has_key_image(&self, key_image: &KeyImage) -> bool {
        self.key_images.contains(key_image)
    }
}
