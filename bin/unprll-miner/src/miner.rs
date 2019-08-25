use futures::{
    Async,
    Future,
    Poll,
    task
};

use common::Block;
use crypto::{
    Digest,
    Hash256,
    RNJC
};

pub struct Miner {
    block: Option<Block>
}

impl Miner {
    pub fn new() -> Miner {
        Miner {
            block: None
        }
    }
    pub fn set_block(&mut self, mut block: Block) {
        block.header.iterations = 0;

        let blob = block.get_mining_blob();
        block.header.hash_checkpoints.push(Hash256::from(RNJC::digest(&blob)));

        self.block = Some(block);
    }
    fn run_pow_step(&mut self) -> bool {
        let block = self.block.take();
        if let Some(mut block) = block {
            let mut hash = *block.header.hash_checkpoints.last().expect("Apparently initialized block doesn't have any hashes").data();

            if compat::difficulty::check_hash_for_difficulty(&hash, 1) {
                block.header.hash_checkpoints.push(Hash256::from(hash));
                self.block = Some(block);
                return true;
            }

            hash = RNJC::digest(&hash);
            block.header.iterations += 1;
            if block.header.iterations % 30 == 0 {
                block.header.hash_checkpoints.push(Hash256::from(hash));
            }

            self.block = Some(block);
        }
        false
    }
}

impl Future for Miner {
    type Item = Block;
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.block.is_none() {
            Ok(Async::NotReady)
        } else if self.run_pow_step() {
            Ok(Async::Ready(self.block.take().expect("PoW step-through was complete, yet there was no block")))
        } else {
            task::current().notify();
            Ok(Async::NotReady)
        }
    }
}
