use serde::{Serialize, Deserialize};

use crypto::{CNFastHash, Digest, Hash256, PublicKey};
use crate::{GetHash, Transaction};

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    pub major_version: u8,
    pub minor_version: u8,
    pub timestamp: u64,
    pub prev_id: Hash256,
    pub miner_specific: PublicKey,
    pub iterations: u32,
    pub hash_checkpoints: Vec<Hash256>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub miner_tx: Transaction,
    pub tx_hashes: Vec<Hash256>
}

impl GetHash for Block {
    fn get_hash(&self) -> Hash256 {
        CNFastHash::digest(&bincode::serialize(self).unwrap())
    }
}
