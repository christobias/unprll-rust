use serde::{Serialize, Deserialize};
use digest::Digest;

use crypto::{CNFastHash, Hash256, Hash256Data, KeyImage, PublicKey, Signature};

use crate::GetHash;

#[derive(Serialize, Deserialize, Debug)]
pub enum TXOutTarget {
    ToKey {
        key: PublicKey
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TXIn {
    Gen {
        height: u64
    },
    FromKey {
        amount: u64,
        key_offsets: Vec<u64>,
        key_image: KeyImage
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TXOut {
    pub amount: u64,
    pub target: TXOutTarget
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TransactionPrefix {
    pub version: usize,
    pub unlock_delta: u16,
    pub inputs: Vec<TXIn>,
    pub outputs: Vec<TXOut>,
    pub extra: Vec<u8>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub prefix: TransactionPrefix,
    pub signatures: Vec<Vec<Signature>>
    // rct_signatures
}

impl GetHash for Transaction {
    fn get_hash(&self) -> Hash256 {
        let mut hashes: Vec<Hash256Data> = Vec::with_capacity(3);
        // Prefix hash
        hashes[0] = CNFastHash::digest(&bincode::serialize(&self.prefix).unwrap());
        // Signatures hash
        hashes[1] = CNFastHash::digest(&bincode::serialize(&self.signatures).unwrap());
        // TODO: RingCT Signatures hash
        // hashes[2] = CNFastHash::digest(&bincode::serialize(&self.signatures).unwrap());

        Hash256::from(CNFastHash::digest(&bincode::serialize(&hashes).unwrap()))
    }
}
