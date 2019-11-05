// TODO: Probably move this to common so libraries don't have to link to wallet

use serde::{
    Serialize,
    Deserialize
};

use crypto::{
    PublicKey
};

mod address_impl;
mod subaddress_impl;

pub trait AddressPrefixes {
    const STANDARD: u64;
    const SUBADDRESS: u64;
    const INTEGRATED: u64;
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum AddressType {
    Standard,
    SubAddress,
    Integrated()
}

impl Default for AddressType {
    fn default() -> Self {
        AddressType::Standard
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Address<TPrefix: AddressPrefixes> {
    #[serde(skip)]
    pub address_type: AddressType,

    pub spend_public_key: PublicKey,
    pub view_public_key: PublicKey,

    marker: std::marker::PhantomData<TPrefix>
}

#[derive(Debug, Eq, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SubAddressIndex(pub u32, pub u32);
