//! Module for handling addresses

use base58_monero::base58::Error as Base58Error;
use failure::Fail;
use serde::{Deserialize, Serialize};

use crypto::{ecc::PointExt, Hash8, PublicKey};

/// Prefixes used to identify an address from its string representation
pub trait AddressPrefixes {
    /// Prefix for a standard address
    const STANDARD: u64;
    /// Prefix for a subaddress
    const SUBADDRESS: u64;
    /// Prefix for an integrated address
    const INTEGRATED: u64;
}

/// Tags for each type of address
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum AddressType {
    /// Standard address
    Standard,
    /// Subaddress
    SubAddress,
    /// Integrated address: Standard address with an included payment ID
    Integrated(Hash8),
}

impl Default for AddressType {
    fn default() -> Self {
        AddressType::Standard
    }
}

/// Wrapper for the set of public keys in an address
#[derive(Clone, Serialize, Deserialize)]
pub struct Address {
    /// Type of address
    #[serde(skip)]
    pub address_type: AddressType,
    /// Public spend key
    pub spend_public_key: PublicKey,
    /// Public view key
    pub view_public_key: PublicKey,
}

/// Error type for Address operations
#[derive(Fail, Debug)]
pub enum Error {
    /// Returned when the address cannot be decoded correctly
    #[fail(display = "Invalid address encoding")]
    InvalidEncoding(#[fail(cause)] Base58Error),

    /// Returned when the address prefix is invalid
    #[fail(display = "Invalid address prefix")]
    InvalidPrefix,
}

impl From<Base58Error> for Error {
    fn from(error: Base58Error) -> Self {
        Self::InvalidEncoding(error)
    }
}

impl Address {
    /// Generate the standard address from the given public keys
    pub fn standard(spend_public_key: PublicKey, view_public_key: PublicKey) -> Self {
        Address {
            address_type: AddressType::Standard,
            spend_public_key,
            view_public_key,
        }
    }

    /// Generate a subaddress from the given public keys
    pub fn subaddress(spend_public_key: PublicKey, view_public_key: PublicKey) -> Self {
        Address {
            address_type: AddressType::SubAddress,
            spend_public_key,
            view_public_key,
        }
    }

    /// Generate an integrated address from the given public keys and payment ID
    pub fn integrated(
        spend_public_key: PublicKey,
        view_public_key: PublicKey,
        payment_id: crypto::Hash8,
    ) -> Self {
        Address {
            address_type: AddressType::Integrated(payment_id),
            spend_public_key,
            view_public_key,
        }
    }

    /// Converts a human readable Cryptonote address into an Address
    pub fn from_address_string<TPrefix: AddressPrefixes>(data: &str) -> Result<Self, Error> {
        let data = base58_monero::decode_check(data)?;

        // Figure out the index where the varint prefix ends
        let mut tag_end = 0;
        for data_byte in &data {
            tag_end += 1;
            // The last byte does not have the most significant bit set
            if data_byte & 0b1000_0000 == 0 {
                break;
            }
        }

        // TODO: While we have deserialization support in bincode_epee now, PublicKey is
        //       serialized with its length, so we need to continue manual deserialization for now
        let spend_public_key = PublicKey::from_slice(&data[(tag_end)..(tag_end + 32)]);
        let view_public_key = PublicKey::from_slice(&data[(tag_end + 32)..(tag_end + 64)]);

        let tag: u64 = varint::deserialize(&data[0..tag_end]);

        if tag == TPrefix::STANDARD {
            Ok(Address::standard(spend_public_key, view_public_key))
        } else if tag == TPrefix::SUBADDRESS {
            Ok(Address::subaddress(spend_public_key, view_public_key))
        } else if tag == TPrefix::INTEGRATED {
            Ok(Address::integrated(
                spend_public_key,
                view_public_key,
                crypto::Hash8::null_hash(),
            ))
        } else {
            Err(Error::InvalidPrefix)
        }
    }

    /// Converts an Address to a human readable Cryptonote address
    pub fn to_address_string<TPrefix: AddressPrefixes>(&self) -> String {
        let mut address = Vec::new();

        // Tag
        let tag = match &self.address_type {
            AddressType::Standard => TPrefix::STANDARD,
            AddressType::SubAddress => TPrefix::SUBADDRESS,
            AddressType::Integrated(payment_id) => TPrefix::INTEGRATED,
        };
        address.extend_from_slice(&varint::serialize(tag));

        // Spend public key
        address.extend_from_slice(&self.spend_public_key.compress().to_bytes());

        // View public key
        address.extend_from_slice(&self.view_public_key.compress().to_bytes());

        // Base58
        base58_monero::encode_check(&address).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_definitions::TestCoin;

    #[test]
    fn it_encodes_addresses_properly() {
        // Unprll Donation address
        let spend_public_key = PublicKey::from_slice(
            &hex::decode("1ed50fe76f3fcd23c16493f8802b04f1c77eace5a54f969cc03dfa5cd3149457")
                .unwrap(),
        );
        let view_public_key = PublicKey::from_slice(
            &hex::decode("36440552e76c9029d22edb4db283b0d9daf2ed21001728248eb4300eaba7f4e0")
                .unwrap(),
        );

        let address = Address::standard(spend_public_key, view_public_key);
        let address: String = address.to_address_string::<TestCoin>();

        assert_eq!(
            address,
            String::from("UNP1Yn4gC4EBfxGByWr4CX8CLnvLRm3ZWEK7BEeiuwYe4SeVpqbRMZxKACWXQ1WCw3P2Zpt68rHZ94sehkF5o8Wn7NAC1PoBzh")
        );
    }

    #[test]
    fn it_decodes_standard_string_addresses_properly() {
        // Unprll Donation address
        let address = Address::from_address_string::<TestCoin>("UNP1Yn4gC4EBfxGByWr4CX8CLnvLRm3ZWEK7BEeiuwYe4SeVpqbRMZxKACWXQ1WCw3P2Zpt68rHZ94sehkF5o8Wn7NAC1PoBzh").unwrap();

        // Address type
        assert_eq!(address.address_type, AddressType::Standard);

        // Spend public key
        assert_eq!(
            hex::encode(address.spend_public_key.compress().as_bytes()),
            "1ed50fe76f3fcd23c16493f8802b04f1c77eace5a54f969cc03dfa5cd3149457"
        );

        // View public key
        assert_eq!(
            hex::encode(address.view_public_key.compress().as_bytes()),
            "36440552e76c9029d22edb4db283b0d9daf2ed21001728248eb4300eaba7f4e0"
        );
    }

    #[test]
    fn it_decodes_subaddress_string_addresses_properly() {
        // Unprll Donation wallet subaddress
        let address = Address::from_address_string::<TestCoin>("UNPStVLMoCzdHGE7EeVNuuWeReeJQXDeEWtRfaCQJ7oJSMr4bYVpreqcP36SjwiCHF86z9bbQecaqcW6yH5ndWx2M6t69dcEoE2").unwrap();

        // Address type
        assert_eq!(address.address_type, AddressType::SubAddress);

        // Spend public key
        assert_eq!(
            hex::encode(address.spend_public_key.compress().as_bytes()),
            "b4c9093fd8e8013eb396e5c0b13cc7819b968c9fb2ae39333c2f078d979d304c"
        );

        // View public key
        assert_eq!(
            hex::encode(address.view_public_key.compress().as_bytes()),
            "be9156eed385d61060ea2d022a779c4b28ecc68ed440517a2a8a0c7b782daa66"
        );
    }
}
