use common::Block;
use crypto::{Digest, Hash256, RNJC};

pub struct Miner {
    block: Option<Block>,
    difficulty: u128,
}

impl Miner {
    pub fn new() -> Miner {
        Miner {
            block: None,
            difficulty: 0,
        }
    }
    pub fn set_block(&mut self, block: Option<Block>) {
        if let Some(mut block) = block {
            block.header.iterations = 0;

            let blob = block.get_mining_blob();
            block
                .header
                .hash_checkpoints
                .push(Hash256::from(RNJC::digest(&blob)));
            self.block = Some(block);
        }
    }
    pub fn take_block(&mut self) -> Option<Block> {
        self.block.take()
    }
    pub fn set_difficulty(&mut self, difficulty: u128) {
        self.difficulty = difficulty;
    }
    pub fn run_pow_step(&mut self) -> bool {
        if let Some(block) = &mut self.block {
            let mut hash = *block
                .header
                .hash_checkpoints
                .last()
                .expect("Apparently initialized block doesn't have any hashes")
                .data();

            if common::difficulty::check_hash_for_difficulty(&hash, self.difficulty) {
                block.header.hash_checkpoints.push(Hash256::from(hash));
                return true;
            }

            hash = RNJC::digest(&hash);
            block.header.iterations += 1;
            if block.header.iterations % 30 == 0 {
                block.header.hash_checkpoints.push(Hash256::from(hash));
            }
        }
        false
    }
}
