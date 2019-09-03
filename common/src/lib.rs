#[macro_use] extern crate log;

use crypto::{
    Digest,
    Hash256,
    PublicKey
};

mod block;
mod checkpoints;
mod transaction;

pub use block::{
    Block,
    BlockHeader
};
pub use checkpoints::Checkpoints;
pub use transaction::{
    Transaction,
    TransactionPrefix,
    TXIn,
    TXOut,
    TXOutTarget
};

pub trait GetHash {
    fn get_hash_blob(&self) -> Vec<u8>;
    fn get_hash(&self) -> Hash256 {
        Hash256::from(crypto::CNFastHash::digest(&self.get_hash_blob()))
    }
}

pub trait PreliminaryChecks<T> {
    fn check(&self, value: &T) -> Result<(), failure::Error>;
}

pub struct Address {
    pub view_public_key: PublicKey,
    pub spend_public_key: PublicKey
}
