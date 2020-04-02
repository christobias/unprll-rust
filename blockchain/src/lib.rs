#![deny(missing_docs)]

//! # Blockchain management
//! This crate handles the blockchain

use std::collections::VecDeque;

use failure::format_err;
use futures::{task::Task, Async, Poll, Stream};
use log::info;

use blockchain_db::{BlockchainDB, Result};
use common::{Block, GetHash, PreliminaryChecks, Transaction};
use crypto::Hash256;

mod config;
mod traits;

pub use config::Config;
pub use traits::EmissionCurve;

/// An interface to the stored blockchain
pub struct Blockchain<TCoin>
where
    // TODO: Wait for trait aliases for simplifying external use
    TCoin: EmissionCurve,
{
    alternative_blocks: Vec<Block>,
    blockchain_db: BlockchainDB,
    coin_definition: TCoin,
    current_task: Option<Task>,
    events: VecDeque<Block>,
}

impl<TCoin> Blockchain<TCoin>
where
    TCoin: EmissionCurve,
{
    /// Creates a new Blockchain with the given configuration
    pub fn new(coin_definition: TCoin, config: &Config) -> Result<Self> {
        let mut blockchain = Blockchain {
            alternative_blocks: Vec::new(),
            blockchain_db: BlockchainDB::new(&config.blockchain_db_config),
            coin_definition,
            current_task: None,
            events: VecDeque::new(),
        };
        if blockchain.blockchain_db.get_block_by_height(0).is_none() {
            // Add the genesis block
            blockchain.add_new_block(Block::genesis())?;
        }
        Ok(blockchain)
    }

    // Blocks
    /// Get blocks from `start` to `end` (inclusive)
    pub fn get_blocks(&self, start: u64, end: u64) -> Vec<Block> {
        let mut vec = Vec::new();
        for i in start..=end {
            if let Some(block) = self.blockchain_db.get_block_by_height(i) {
                vec.push(block);
            }
        }
        vec
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
        // Do all prechecks
        self.check(&block)?;

        // Add the block
        // TODO: Add transactions once the mempool is done
        self.blockchain_db.add_block(block.clone(), Vec::new())?;

        // Notify any pending tasks
        if let Some(task) = &self.current_task {
            task.notify();
            self.events.push_back(block.clone());
        }

        // Print a log message for confirmation
        let (height, _) = self.get_tail()?;
        info!(
            "Added new block:\tBlock ID: {}\tBlock Height: {}",
            block.get_hash(),
            height
        );
        Ok(())
    }

    /// Gets a block from the blockchain
    pub fn get_block(&self, id: &Hash256) -> Option<Block> {
        self.blockchain_db.get_block_by_hash(id)
    }

    /// Gets the main chain's tail
    ///
    /// # Returns
    /// A `(block height, Block)` tuple
    pub fn get_tail(&self) -> Result<(u64, Block)> {
        self.blockchain_db.get_tail()
    }

    // Transactions
    /// Gets a transaction with the given txid from confirmed transactions
    pub fn get_transaction(&self, txid: &Hash256) -> Option<Transaction> {
        self.blockchain_db.get_transaction(txid)
    }
}

impl<TCoin: EmissionCurve> PreliminaryChecks<Block> for Blockchain<TCoin> {
    fn check(&self, block: &Block) -> Result<()> {
        // Do the blockchain DB prechecks
        self.blockchain_db.check(block)?;

        // The coinbase transaction must have only one input and output
        if block.miner_tx.prefix.inputs.len() != 1 {
            return Err(format_err!(
                "Block {}'s coinbase transaction does not have exactly one input!",
                block.get_hash()
            ));
        }

        if block.miner_tx.prefix.outputs.len() != 1 {
            return Err(format_err!(
                "Block {}'s coinbase transaction does not have exactly one output!",
                block.get_hash()
            ));
        }

        // The coinbase amount must match the coin's emission curve
        if block.miner_tx.prefix.outputs[0].amount
            != self
                .coin_definition
                .get_block_reward(block.header.major_version)?
        {
            return Err(format_err!(
                "Block {}'s coinbase transaction does not follow the coin's emission curve!",
                block.get_hash()
            ));
        }

        Ok(())
    }
}

impl<TCoin> Stream for Blockchain<TCoin>
where
    TCoin: EmissionCurve,
{
    type Item = Block;
    type Error = ();

    // TODO: This has to become a read-only reference to blockchain to make
    //       sure we don't block any other readers. We are only ever going
    //       to use read-only methods on the actual blockchain
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.current_task = Some(futures::task::current());
        if let Some(event) = self.events.pop_front() {
            return Ok(Async::Ready(Some(event)));
        }
        Ok(Async::NotReady)
    }
}
