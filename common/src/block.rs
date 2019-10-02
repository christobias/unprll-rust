use std::convert::TryFrom;

use serde::{Serialize, Deserialize};

use crypto::{
    Hash256,
    Hash256Data,
    PublicKey
};
use crate::{
    GetHash,
    Transaction,
    TransactionPrefix,
    TXIn,
    TXOut,
    TXOutTarget
};

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

impl Block {
    pub fn genesis() -> Block {
        Block {
            header: BlockHeader {
                major_version: 1,
                minor_version: 9,
                timestamp: 300,
                prev_id: Hash256::null_hash(),
                miner_specific: PublicKey::from_slice(&[0; 32]),
                iterations: 0,
                hash_checkpoints: vec![
                    Hash256::try_from("302f625d28c819b2bcaae7e4d73dc4152c4f201b1951e221547b0d75e9d636ab").unwrap(),
                    Hash256::try_from("302f625d28c819b2bcaae7e4d73dc4152c4f201b1951e221547b0d75e9d636ab").unwrap()
                ]
            },
            miner_tx: Transaction {
                prefix: TransactionPrefix {
                    extra: hex::decode("017767aafcde9be00dcfd098715ebcf7f410daebc582fda69d24a28e9d0bc890d1").unwrap(),
                    inputs: vec![
                        TXIn::Gen {
                            height: 0
                        }
                    ],
                    outputs: vec![
                        TXOut {
                            amount: 17_590_000_000_000,
                            target: TXOutTarget::ToKey {
                                key: PublicKey::from_slice(&hex::decode("9b2e4c0281c0b02e7c53291a94d1d0cbff8883f8024f5142ee494ffbbd088071").unwrap())
                            }
                        }
                    ],
                    unlock_delta: 3,
                    version: 1
                },
                signatures: Vec::new()
            },
            tx_hashes: Vec::new()
        }
    }

    /// Gets the "mining blob" for a given block
    ///
    /// It is used to generate the proof-of-work and thus doesn't serialize a few fields (notably
    /// those used for the proof-of-work)
    pub fn get_mining_blob(&self) -> Vec<u8> {
        let mut blob = Vec::with_capacity(std::mem::size_of_val(&self));

        // Major and minor versions
        blob.extend_from_slice(&bincode_epee::serialize(&self.header.major_version).unwrap());
        blob.extend_from_slice(&bincode_epee::serialize(&self.header.minor_version).unwrap());

        // Rounded timestamp
        let mut timestamp = self.header.timestamp;
        timestamp = timestamp - (timestamp % 600) + 300;
        blob.extend_from_slice(&bincode_epee::serialize(&timestamp).unwrap());

        // Previous block ID
        blob.extend_from_slice(&bincode_epee::serialize(&self.header.prev_id).unwrap());

        // Custom serialization for miner specific
        for byte in self.header.miner_specific.as_bytes() {
            blob.extend_from_slice(&bincode_epee::serialize(&byte).unwrap());
        }

        // Transaction root hash
        if !self.tx_hashes.is_empty() {
            let tx_hashes = self.tx_hashes.clone();
            blob.extend_from_slice(crypto::tree_hash(&tx_hashes).data());
        } else {
            blob.extend_from_slice(Hash256::null_hash().data());
        }

        // # of transactions
        blob.extend_from_slice(&bincode_epee::serialize(&self.tx_hashes.len()).unwrap());

        blob
    }
}

impl GetHash for Block {
    fn get_hash_blob(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(std::mem::size_of_val(self));

        // Major and minor versions
        vec.extend_from_slice(&bincode_epee::serialize(&self.header.major_version).unwrap());
        vec.extend_from_slice(&bincode_epee::serialize(&self.header.minor_version).unwrap());

        // Timestamp
        vec.extend_from_slice(&bincode_epee::serialize(&self.header.timestamp).unwrap());

        // Previous block ID
        vec.extend_from_slice(&bincode_epee::serialize(&self.header.prev_id).unwrap());

        // Miner specific
        vec.extend_from_slice(&bincode_epee::serialize(&Hash256Data::from(self.header.miner_specific.to_bytes())).unwrap());

        // Proof of Work
        vec.extend_from_slice(&bincode_epee::serialize(&self.header.iterations).unwrap());
        vec.extend_from_slice(&bincode_epee::serialize(&self.header.hash_checkpoints).unwrap());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genesis_has_the_right_id() {
        let block = Block::genesis();
        assert_eq!(
            hex::encode(block.get_hash().data()),
            "7d491759c7534ca5a8be62ec7fa34dc939659f5afd4b4f1da2c671a84773cedc"
        );
    }
}
