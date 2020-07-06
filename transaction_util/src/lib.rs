#![deny(missing_docs)]
//! Utilities for handling transactions

use crypto::{PublicKey, SecretKey};
use ringct::DestinationCommitmentPair;

mod account_keys;
pub mod address;
pub mod payment_id;
pub mod subaddress;
#[cfg(test)]
mod test_definitions;

pub use account_keys::AccountKeys;
use address::{Address, AddressPrefixes};
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
    pub real_output_index: u64,
    /// Set of transaction public keys for the real output
    pub real_output_tx_public_keys: Vec<PublicKey>,
    /// Index of the subaddress to which the real output was paid to
    pub subaddress_index: SubAddressIndex,
}

/// Destination type
pub enum TXDestinationType<TCoin: AddressPrefixes> {
    /// Output amount is towards another address
    PayToAddress(Address<TCoin>),
    /// Output amount is to be sent back to us as change
    Change(SubAddressIndex)
}

/// A destination entry for a given transaction
pub struct TXDestination<TCoin: AddressPrefixes> {
    /// Amount being paid to this destination
    pub amount: u64,
    /// Type of destination
    pub destination_type: TXDestinationType<TCoin>,
}

