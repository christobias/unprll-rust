#![deny(missing_docs)]

//! # Blockchain management
//! This crate handles the blockchain

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

/// An interface to the stored blockchain
pub struct Blockchain {
    alternative_blocks: Vec<Block>,
    blockchain_db: BlockchainDB,
    current_task: Option<Task>,
    events: VecDeque<Block>
}

impl Blockchain {
    /// Creates a new Blockchain with the given configuration
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
    /// Get `count` blocks from `start`
    pub fn get_blocks(&self, start: u64, count: u64) -> Option<Vec<Block>> {
        let mut vec = Vec::new();
        for i in start..(start+count) {
            vec.push(self.blockchain_db.get_block_by_height(i)?);
        }
        Some(vec)
    }

    /// Get a reference to alternative blocks received from other peers
    pub fn get_alternative_blocks(&self) -> &Vec<Block> {
        &self.alternative_blocks
    }

    /// Adds a new block to the main chain
    ///
    /// The block must satisfy the blockchain database's preliminary checks (another block doesn't
    /// exist at the given height already, all transactions in the block don't exist already, must
    /// connect to the current chain's tail) and further it must have a valid proof-of-work (as
    /// determined by the coin)
    ///
    /// # Returns
    /// An empty tuple if the block was added successfully
    ///
    /// # Errors
    /// If any of the pre-checks fail
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

    /// Gets a block from the blockchain
    pub fn get_block(&self, id: &Hash256) -> Option<Block> { self.blockchain_db.get_block_by_hash(id) }

    /// Gets the main chain's tail
    ///
    /// # Returns
    /// A `(block height, Block)` tuple
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
