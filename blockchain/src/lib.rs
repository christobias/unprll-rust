use std::collections::VecDeque;

use log::info;

use blockchain_db::{
    BlockchainDB,
    Result
};
use common::{
    Block,
    GetHash,
    PreliminaryChecks
};
use crypto::{
    Hash256
};
use futures::{
    Async,
    Poll,
    Stream,
    task::Task
};

mod config;
pub use config::Config;

pub struct Blockchain {
    alternative_blocks: Vec<Block>,
    blockchain_db: BlockchainDB,
    current_task: Option<Task>,
    events: VecDeque<Block>
}

impl Blockchain {
    pub fn new(config: &Config) -> Result<Self> {
        let mut blockchain = Blockchain {
            alternative_blocks: Vec::new(),
            blockchain_db: BlockchainDB::new(&config.blockchain_db_config),
            current_task: None,
            events: VecDeque::new()
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

        if let Some(task) = &self.current_task {
            task.notify();
            self.events.push_back(block.clone());
        }

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

impl Stream for Blockchain {
    type Item = Block;
    type Error = ();

    // TODO: This has to become a read-only reference to blockchain to make
    //       sure we don't block any other readers. We are only ever going
    //       to use read-only methods on the actual blockchain
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.current_task = Some(futures::task::current());
        if let Some(event) = self.events.pop_front() {
            return Ok(Async::Ready(Some(event)))
        }
        Ok(Async::NotReady)
    }
}
