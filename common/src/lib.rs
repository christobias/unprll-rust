#![deny(missing_docs)]
//! Common definitions and traits across crates

mod block;
mod traits;

/// Functions for determining default directories
pub mod data_dir;

/// Functions for proof-of-work difficulty verification
pub mod difficulty;
mod transaction;

pub use block::{Block, BlockHeader};
pub use traits::{GetHash, PreliminaryChecks};
pub use transaction::{TXExtra, TXIn, TXNonce, TXOut, TXOutTarget, Transaction, TransactionPrefix};
