use crypto::{PublicKey, KeyImage};
use common::{Address, Config};
use blockchain_db::{BlockchainDB, Result};
use common::Block;
use crypto::Hash256;

pub struct Blockchain {
    blockchain_db: Box<dyn BlockchainDB + Sync + Send>
}

impl Blockchain {
    pub fn new(config: &Config) -> Result<Self> {
        Ok(Blockchain {
            blockchain_db: match config.db_type.as_ref() {
                "memory" => Box::new(blockchain_db::BlockchainMemDB::new()),
                "lmdb" => Box::new(blockchain_db::BlockchainLMDB::new(&Box::from(std::path::Path::new("")))?),
                _ => panic!("Unknown DB type!")
            }
        })
    }

    // Blocks
    pub fn get_blocks(&self, start: u64, count: u64) -> Result<Vec<Block>> {
        let mut vec = Vec::new();
        for i in start..(start+count) {
            vec.push(self.blockchain_db.get_block_by_height(i)?);
        }
        Ok(vec)
    }

    pub fn get_alternative_blocks() -> Result<Vec<Block>> {
        unimplemented!()
    }

    pub fn get_block(&self, id: &Hash256) -> Result<Block> {
        self.blockchain_db.get_block_by_hash(id)
    }

    pub fn create_block_template(_miner_address: Address) -> Block {
        unimplemented!()
    }

    pub fn add_new_block(&mut self, block: Block) -> Result<()> {
        self.blockchain_db.add_block(block, 0, 0, 0, Vec::new())
    }

    // Transactions
    pub fn have_tx(&self, id: &Hash256) -> bool {
        self.blockchain_db.get_transaction(id).is_ok()
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

    pub fn find_blockchain_supplement(short_history: Vec<Hash256>) -> Result<Vec<Hash256>> {
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
