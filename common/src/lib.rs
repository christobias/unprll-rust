#![deny(missing_docs)]
//! Common definitions and traits across crates

// #[macro_use] extern crate log;

use crypto::{
    Digest,
    Hash256
};

mod block;
// mod checkpoints;

/// Functions for determining default directories
pub mod data_dir;

/// Functions for proof-of-work difficulty verification
pub mod difficulty;
mod transaction;

pub use block::{
    Block,
    BlockHeader
};
// pub use checkpoints::Checkpoints;
pub use transaction::{
    Transaction,
    TransactionPrefix,
    TXExtra,
    TXIn,
    TXOut,
    TXOutTarget
};

/// Gets a hash of an implementor (usually the Keccak finalist (CNFastHash) hash of the
/// implementor's binary serialization)
pub trait GetHash {
    /// Gets a raw byte-wise representation of the implementor ready for hashing
    fn get_hash_blob(&self) -> Vec<u8>;

    /// Gets the hash of the implementor
    ///
    /// This hash serves as the ID of the implementor and can thus be adapted for different ID
    /// constructions
    fn get_hash(&self) -> Hash256 {
        Hash256::from(crypto::CNFastHash::digest(&self.get_hash_blob()))
    }
}

/// Trait for specifying that the implementor requires input data to satisfy certain conditions
///
/// Users can use this trait to do certain checks before more expensive checks
pub trait PreliminaryChecks<T> {
    /// Checks a given input according to the implementor's prerequisites
    ///
    /// # Returns
    /// An empty tuple if the input passes the prerequisites
    ///
    /// # Errors
    /// If the input doesn't satisfy the implementor's prerequisites
    fn check(&self, value: &T) -> Result<(), failure::Error>;
}
