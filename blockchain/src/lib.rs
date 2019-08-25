use log::info;

use blockchain_db::{BlockchainDB, Result};
use common::{
    Block,
    Config,
    GetHash,
    PreliminaryChecks
};
use crypto::{
    Hash256
};

pub struct Blockchain {
    alternative_blocks: Vec<Block>,

    blockchain_db: BlockchainDB
}

impl Blockchain {
    pub fn new(config: &Config) -> Result<Self> {
        let mut blockchain = Blockchain {
            alternative_blocks: Vec::new(),
            blockchain_db: BlockchainDB::new(config)
        };
        if blockchain.blockchain_db.get_block_by_height(0).is_none() {
            // Add the genesis block
            blockchain.add_new_block(Block::genesis())?;
        }
        Ok(blockchain)
    }

    // Blocks
    pub fn get_blocks(&self, start: u64, count: u64) -> Option<Vec<Block>> {
        let mut vec = Vec::new();
        for i in start..(start+count) {
            vec.push(self.blockchain_db.get_block_by_height(i)?);
        }
        Some(vec)
    }

    pub fn get_alternative_blocks(&self) -> &Vec<Block> {
        &self.alternative_blocks
    }

    pub fn add_new_block(&mut self, block: Block) -> Result<()> {
        self.blockchain_db.check(&block)?;
        self.blockchain_db.add_block(block.clone(), Vec::new())?;
        let (height, _) = self.get_tail()?;
        info!("Added new block:\tBlock ID: {}\tBlock Height: {}", block.get_hash(), height);
        Ok(())
    }

    // Other
    pub fn get_short_chain_history() -> Vec<Hash256> {
        unimplemented!()
    }

    pub fn find_blockchain_supplement(_short_history: Vec<Hash256>) -> Result<Vec<Hash256>> {
        unimplemented!()
    }

    // pub fn have_tx(&self, id: &Hash256) -> bool { self.blockchain_db.get_transaction(id).is_some() }
    // pub fn is_keyimage_spent(&self, key_image: &KeyImage) -> bool { self.blockchain_db.has_key_image(key_image) }
    pub fn get_block(&self, id: &Hash256) -> Option<Block> { self.blockchain_db.get_block_by_hash(id) }
    pub fn get_tail(&self) -> Result<(u64, Block)> { self.blockchain_db.get_tail() }
}
