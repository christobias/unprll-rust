extern crate structopt;

use crypto::Hash256;
use crypto::{PublicKey};

mod block;
// mod checkpoints;
mod config;
mod transaction;

pub use block::Block;
// pub use checkpoints::Checkpoints;
pub use config::Config;
pub use transaction::{Transaction,TransactionPrefix,TXIn,TXOut,TXOutTarget};

pub trait GetHash {
    fn get_hash(&self) -> Hash256;
}

pub struct Address {
    pub view_public_key: PublicKey,
    pub spend_public_key: PublicKey
}
