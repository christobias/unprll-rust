#![deny(missing_docs)]
//! Common definitions and traits across crates

#[macro_use]
extern crate failure;

mod block;
mod traits;

/// Functions for determining default directories
pub mod data_dir;

mod address;
/// Functions for proof-of-work difficulty verification
pub mod difficulty;
mod transaction;

pub use address::{Address, AddressPrefixes, AddressType, SubAddressIndex};
pub use block::{Block, BlockHeader};
pub use traits::{GetHash, PreliminaryChecks};
pub use transaction::{TXExtra, TXIn, TXOut, TXOutTarget, Transaction, TransactionPrefix};
