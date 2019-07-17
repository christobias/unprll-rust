extern crate blockchain_db;
extern crate common;
extern crate crypto;

use crypto::{PublicKey, KeyImage};
use common::{Address, Config};
use blockchain_db::{BlockchainDB, Error};
use common::Block;
use crypto::Hash256;

pub struct Blockchain {
    blockchain_db: Box<dyn BlockchainDB + Send>
}

impl Blockchain {
    pub fn new(config: &Config) -> Result<Self, Error> {
        Ok(Blockchain {
            blockchain_db: Box::new(match config.db_type.as_ref() {
                "memory" => blockchain_db::BlockchainMemDB::new(),
                _ => panic!("Unknown DB type!")
            })
        })
    }

    // Blocks
    pub fn get_blocks(start: u64, count: u64) -> Result<Vec<Block>, ()> {
        unimplemented!()
    }

    pub fn get_alternative_blocks() -> Result<Vec<Block>, ()> {
        unimplemented!()
    }

    pub fn get_block(id: Hash256) -> Result<Block, ()> {
        unimplemented!()
    }

    pub fn get_block_id(height: u64) -> Result<Hash256, ()> {
        unimplemented!()
    }

    pub fn create_block_template(miner_address: Address) -> Block {
        unimplemented!()
    }

    pub fn add_new_block(block: Block) -> Result<(), ()> {
        unimplemented!()
    }

    // Transactions
    pub fn have_tx(id: Hash256) -> bool {
        unimplemented!()
    }

    pub fn is_keyimage_spent(key_image: KeyImage) -> bool {
        unimplemented!()
    }

    // Outputs
    pub fn get_num_mature_rct_outputs() -> u64 {
        unimplemented!()
    }

    pub fn get_rct_output_key(global_index: u64) -> PublicKey {
        unimplemented!()
    }

    pub fn get_outputs() {
        unimplemented!()
    }

    pub fn get_output(global_index: u64) {
        unimplemented!()
    }

    pub fn get_rct_output_distribution(from: u64, to: u64) -> Vec<u64> {
        unimplemented!()
    }

    // Other
    pub fn get_tail() -> (u64, Block) {
        unimplemented!()
    }

    pub fn get_short_chain_history() -> Vec<Hash256> {
        unimplemented!()
    }

    pub fn find_blockchain_supplement(short_history: Vec<Hash256>) -> Result<Vec<Hash256>, ()> {
        unimplemented!()
    }

    pub fn reset() {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
