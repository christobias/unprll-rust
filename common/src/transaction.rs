use serde::{
    Serialize,
    Deserialize
};
use digest::Digest;

use crypto::{CNFastHash, Hash256, Hash256Data, KeyImage, PublicKey, Signature};

use crate::GetHash;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TXOutTarget {
    ToKey {
        key: PublicKey
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TXOut {
    pub amount: u64,
    pub target: TXOutTarget
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TransactionPrefix {
    pub version: usize,
    pub unlock_delta: u16,
    pub inputs: Vec<TXIn>,
    pub outputs: Vec<TXOut>,
    pub extra: Vec<u8>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub prefix: TransactionPrefix,
    pub signatures: Vec<Vec<Signature>>
    // rct_signatures
}

impl GetHash for Transaction {
    fn get_hash_blob(&self) -> Vec<u8> {
        let mut vec = Vec::with_capacity(std::mem::size_of_val(&self));

        // Tx version
        vec.extend_from_slice(&bincode_epee::serialize(&self.prefix.version).unwrap());

        // Unlock delta
        vec.extend_from_slice(&bincode_epee::serialize(&self.prefix.unlock_delta).unwrap());

        // Inputs
        vec.extend_from_slice(&bincode_epee::serialize(&self.prefix.inputs.len()).unwrap());
        for input in &self.prefix.inputs {
            match input {
                TXIn::Gen { height } => {
                    // Enum tag
                    vec.extend_from_slice(&bincode_epee::serialize(&0xffu8).unwrap());

                    // Input
                    vec.extend_from_slice(&bincode_epee::serialize(height).unwrap());
                },
                _ => unimplemented!()
            }
        }

        // Outputs
        vec.extend_from_slice(&bincode_epee::serialize(&self.prefix.outputs.len()).unwrap());
        for output in &self.prefix.outputs {
            // Amount
            vec.extend_from_slice(&bincode_epee::serialize(&output.amount).unwrap());

            // Target
            match output.target {
                TXOutTarget::ToKey { key } => {
                    // Enum tag
                    vec.extend_from_slice(&bincode_epee::serialize(&0x02).unwrap());

                    // Public Key
                    vec.extend_from_slice(&bincode_epee::serialize(&Hash256Data::from(key.to_bytes())).unwrap());
                }
            }
        }

        // Extra
        vec.extend_from_slice(&bincode_epee::serialize(&self.prefix.extra).unwrap());

        // Signatures
        if !self.signatures.is_empty() {
            vec.extend_from_slice(&bincode_epee::serialize(&self.signatures).unwrap());
        }
        vec
    }
    fn get_hash(&self) -> Hash256 {
        if self.prefix.version == 1 {
            return Hash256::from(CNFastHash::digest(&self.get_hash_blob()));
        }
        let mut hashes: Vec<Hash256Data> = Vec::with_capacity(3);
        // Prefix hash
        hashes.push(CNFastHash::digest(&bincode::serialize(&self.prefix).unwrap()));
        // Signatures hash
        hashes.push(CNFastHash::digest(&bincode::serialize(&self.signatures).unwrap()));
        // TODO: RingCT Signatures hash
        // hashes[2] = CNFastHash::digest(&bincode::serialize(&self.signatures).unwrap());

        Hash256::from(CNFastHash::digest(&bincode::serialize(&hashes).unwrap()))
    }
}
