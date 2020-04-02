// TODO: Probably move this to common so libraries don't have to link to wallet

use serde::{Deserialize, Serialize};

use crypto::PublicKey;

mod address_impl;
mod subaddress_impl;

/// Prefixes used to identify an address from its string representation
pub trait AddressPrefixes {
    /// Prefix for a standard address
    const STANDARD: u64;
    /// Prefix for a subaddress
    const SUBADDRESS: u64;
    /// Prefix for an integrated address
    const INTEGRATED: u64;
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum AddressType {
    Standard,
    SubAddress,
    Integrated(),
}

impl Default for AddressType {
    fn default() -> Self {
        AddressType::Standard
    }
}

/// Wrapper for the set of public keys in an address
#[derive(Clone, Serialize, Deserialize)]
pub struct Address<TPrefix: AddressPrefixes> {
    /// Type of address
    #[serde(skip)]
    pub address_type: AddressType,

    /// Public spend key
    pub spend_public_key: PublicKey,
    /// Public view key
    pub view_public_key: PublicKey,

    marker: std::marker::PhantomData<TPrefix>,
}

/// Tuple of (major, minor) index for a subaddress
#[derive(Debug, Eq, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SubAddressIndex(pub u32, pub u32);
