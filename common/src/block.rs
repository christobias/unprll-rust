use serde::{Serialize, Deserialize};

use crypto::{CNFastHash, Digest, Hash256, PublicKey};
use crate::{GetHash, Transaction};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BlockHeader {
    pub major_version: u8,
    pub minor_version: u8,
    pub timestamp: u64,
    pub prev_id: Hash256,
    pub miner_specific: PublicKey,
    pub iterations: u32,
    pub hash_checkpoints: Vec<Hash256>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub miner_tx: Transaction,
    pub tx_hashes: Vec<Hash256>
}

impl GetHash for Block {
    fn get_hash_blob(&self) -> Vec<u8> {
        // Block timestamp fuzzing
        let mut header = self.header.clone();
        header.timestamp = header.timestamp - (header.timestamp % 600) + 300;

        let mut vec = Vec::with_capacity(std::mem::size_of_val(self));
        // Serialize the header
        vec.extend_from_slice(&bincode_epee::serialize(&header).unwrap());

        // Get and serialize the tree hash of the block (including the miner transaction)
        let mut hashes = vec!{self.miner_tx.get_hash()};
        hashes.extend_from_slice(&self.tx_hashes);
        vec.extend_from_slice(crypto::tree_hash(&hashes).data());

        // Serialize the number of transactions in the block (including the miner transaction)
        vec.extend_from_slice(&bincode_epee::serialize(&(self.tx_hashes.len() + 1)).unwrap());

        // Prepend the length of the blob
        bincode_epee::serialize(&vec).unwrap()
    }
}
