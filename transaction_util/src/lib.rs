#![deny(missing_docs)]
//! Utilities for handling transactions

use crypto::{PublicKey, SecretKey};
use ringct::DestinationCommitmentPair;

mod account_keys;
pub mod address;
mod derivation;
pub mod payment_id;
pub mod subaddress;
#[cfg(test)]
mod test_definitions;
pub mod tx_construction;
pub mod tx_scanning;

pub use account_keys::AccountKeys;
use address::Address;
pub use derivation::Derivation;
use subaddress::SubAddressIndex;

/// A source entry for a given transaction
pub struct TXSource {
    /// Amount obtained from the given output in a previous transaction
    pub amount: u64,
    /// Mask value used to hide the amount
    pub amount_mask: SecretKey,
    /// A set of outputs used for hiding the real input in the transaction
    pub outputs: Vec<(u64, DestinationCommitmentPair)>,
    /// The position of the real output being spent among the set of outputs
    ///
    /// Used for the mixin ring
    pub real_output_index: u64,
    /// The position of the real output among the set of outputs in its parent transaction
    ///
    /// Used for the key image scalar derivation
    pub real_output_tx_index: u64,
    /// Set of transaction public keys for the real output
    pub real_output_tx_public_keys: Vec<PublicKey>,
    /// Index of the subaddress to which the real output was paid to
    pub subaddress_index: SubAddressIndex,
}

/// Destination type
pub enum TXDestinationType {
    /// Output amount is towards another address
    PayToAddress(Address),
    /// Output amount is to be sent back to us as change
    Change(SubAddressIndex),
}

/// A destination entry for a given transaction
pub struct TXDestination {
    /// Amount being paid to this destination
    pub amount: u64,
    /// Type of destination
    pub destination_type: TXDestinationType,
}
