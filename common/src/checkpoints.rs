use std::collections::HashMap;
use std::convert::TryFrom;

use crypto::Hash256;

pub struct Checkpoint {
    pub height: u64,
    pub hash: Hash256
}

pub struct Checkpoints {
    checkpoints: HashMap<u64, Hash256>
}

impl Checkpoints {
    pub fn new() -> Checkpoints {
        Checkpoints {
            checkpoints: HashMap::new()
        }
    }
    pub fn for_network(nettype: &str) -> Checkpoints {
        let mut c = Checkpoints::new();
        c.add_checkpoint(0,     Hash256::from(&hex::decode("7d491759c7534ca5a8be62ec7fa34dc939659f5afd4b4f1da2c671a84773cedc").unwrap())).unwrap();
        c
    }
    pub fn add_checkpoint(&mut self, height: u64, hash: Hash256) -> Result<(), ()> {
        // If we have the checkpoint already with a different hash, return an error
        if self.checkpoints.contains_key(&height) && self.checkpoints[&height] != hash {
            return Err(());
        }
        self.checkpoints.insert(height, hash);
        Ok(())
    }
    pub fn in_checkpoint_zone(&self, height: u64) -> bool {
        !self.checkpoints.len() == 0 && height <= *self.checkpoints.iter().last().unwrap().0
    }
    pub fn check_block(&self, height: &u64, hash: &Hash256) -> Result<bool, ()> {
        if !self.checkpoints.contains_key(height) {
            return Ok(false);
        } else if self.checkpoints[height] == *hash {
            debug!("CHECKPOINT PASSED FOR HEIGHT {} {}", height, hash);
            return Ok(true);
        }
        warn!("CHECKPOINT FAILED FOR HEIGHT {}. EXPECTED HASH: {}, , FETCHED HASH: {}", height, self.checkpoints[height], hash);
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut c = Checkpoints::new();
        assert!(!c.in_checkpoint_zone(1));
        c.add_checkpoint(100, Hash256::try_from("1111111111111111111111111111111111111111111111111111111111111111").unwrap()).unwrap();
        // assert!(c.in_checkpoint_zone(1));
    }
}
