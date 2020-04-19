#![deny(missing_docs)]
//! Interfaces for communicating to the backing database of a blockchain
//!
//! The blockchain DB maintains the main chain and ensures the following
//! assumptions hold true:
//!
//! 1. The chain starts with a genesis block (null prev_id, height 0)
//! 2. New blocks connect to the current main chain's tail with increasing heights
//! 3. There are no duplicate transactions
//! 4. There are no transactions that use spent key images
//!
//! The semantics of a valid block or transaction in a given chain is handled by
//! the `Blockchain` struct for a given network

#[macro_use]
extern crate failure;

use common::{Block, GetHash, PreliminaryChecks, TXIn, Transaction};
use crypto::{Hash256, KeyImage};

mod config;
mod error;
// mod lmdb;
mod mem;

pub use config::Config;
pub use error::{Error, Result};

/// Manages communication between the database and the rest of the application.
trait BlockchainDBDriver {
    // DB Operations
    fn is_read_only(&self) -> bool;
    fn sync(&self) -> Result<()>;
    fn set_safe_sync_mode(&self, state: bool);
    fn reset(&mut self);
    fn size(&self) -> u64;
    fn fixup(&self);

    // Block
    fn add_block(&mut self, block: Block) -> Result<()>;
    fn get_block_by_height(&self, height: u64) -> Option<Block>;
    fn get_block_by_hash(&self, block_id: &Hash256) -> Option<Block>;

    // Zero index height, for consistency
    fn get_tail(&self) -> Option<(u64, Block)>;
    fn pop_block(&mut self) -> Option<Block>;

    fn get_cumulative_difficulty(&self) -> u64;

    // Confirmed Transactions
    fn add_transaction(&mut self, transaction: Transaction) -> Result<()>;
    fn get_transaction(&self, id: &Hash256) -> Option<Transaction>;

    // Key Image
    fn add_key_image(&mut self, key_image: KeyImage) -> Result<()>;
    fn has_key_image(&self, key_image: &KeyImage) -> bool;
}

/// # Blockchain database
///
/// This struct interfaces between users and db "drivers" and maintains the following policy:
///
/// 1. Adding a new block requires that it, it's transactions, and all key images in each
///    transaction don't exist already
/// 2. Each block connects to its parent
/// 3. Confirmed transactions can only be added via blocks
pub struct BlockchainDB {
    db: Box<dyn BlockchainDBDriver + Sync + Send>,
}

impl BlockchainDB {
    /// Creates a new BlockchainDB with the specified configuration
    pub fn new(config: &Config) -> Self {
        BlockchainDB {
            db: match config.db_type.as_ref() {
                "memory" => Box::new(mem::BlockchainMemDB::new(config)),
                _ => panic!(),
            },
        }
    }

    /// Adds a new block to the chain
    ///
    /// The new block must satisfy the following prerequisites:
    /// 1. The new block must connect to our current chain tail
    /// 2. That block doesn't exist already
    /// 3. All transactions in the block don't exist already
    /// 4. All key images in the block don't exist already
    pub fn add_block(&mut self, block: Block, transactions: Vec<Transaction>) -> Result<()> {
        // Do preliminary checks
        self.check(&block)?;
        for tx in transactions.iter() {
            self.check(tx)?;
        }
        // Then insert everything
        self.db.add_transaction(block.miner_tx.clone())?;
        for tx in transactions.into_iter() {
            for input in tx.prefix.inputs.iter() {
                if let TXIn::FromKey { key_image, .. } = input {
                    self.db.add_key_image(*key_image)?;
                }
            }
            self.db.add_transaction(tx)?;
        }

        self.db.add_block(block)
    }

    // Passthrough

    /// Gets the block at the given height
    pub fn get_block_by_height(&self, height: u64) -> Option<Block> {
        self.db.get_block_by_height(height)
    }
    /// Gets the block with the given hash
    pub fn get_block_by_hash(&self, hash: &Hash256) -> Option<Block> {
        self.db.get_block_by_hash(hash)
    }
    /// Gets the current chain tail
    pub fn get_tail(&self) -> Option<(u64, Block)> {
        self.db.get_tail()
    }
    /// Gets the transaction with the given txid
    pub fn get_transaction(&self, txid: &Hash256) -> Option<Transaction> {
        self.db.get_transaction(txid)
    }
}

impl PreliminaryChecks<Block, Error> for BlockchainDB {
    fn check(&self, block: &Block) -> Result<()> {
        let block_id = block.get_hash();
        // Verify that:

        // 1. We have a genesis input
        let height = if let TXIn::Gen(h) = block.miner_tx.prefix.inputs[0] {
            Ok(h)
        } else {
            Err(Error::InvalidHeight)
        }?;

        // 2. This new block connects to our existing chain
        //                    - or -
        //    This block's height is 0 and connects to null
        let tail = self.db.get_tail();
        if let Some((_, tail_block)) = tail {
            if tail_block.get_hash() != block.header.prev_id {
                return Err(Error::DoesNotConnect);
            }
        } else if block.header.prev_id != Hash256::null_hash() {
            return Err(Error::DoesNotConnect);
        } else if height != 0 {
            return Err(Error::InvalidHeight);
        }

        // 3. We don't have that block already
        // TODO: This might be redundant because having the same block would imply
        //       it doesn't connect to the chain tail
        if self.db.get_block_by_hash(&block_id).is_some() {
            return Err(Error::Exists);
        }

        // 4. It doesn't overwrite a block on an existing height
        if self.db.get_block_by_height(height).is_some() {
            return Err(Error::Exists);
        }

        Ok(())
    }
}

impl PreliminaryChecks<Transaction, Error> for BlockchainDB {
    fn check(&self, transaction: &Transaction) -> Result<()> {
        let txid = transaction.get_hash();
        // 5. We don't have any of the transactions already
        if self.db.get_transaction(&txid).is_some() {
            return Err(Error::Exists);
        }

        for input in transaction.prefix.inputs.iter() {
            if let TXIn::FromKey { key_image, .. } = input {
                // 6. We don't have any of the key images already
                if self.db.has_key_image(key_image) {
                    return Err(Error::Exists);
                }
            }
        }
        Ok(())
    }
}
