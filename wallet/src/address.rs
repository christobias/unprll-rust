use std::convert::{
    Into,
    TryFrom
};

use serde::{
    Serialize,
    Deserialize
};

use crypto::{
    PublicKey
};

pub trait AddressPrefixConfig {
    const STANDARD: u64;
    const SUBADDRESS: u64;
    const INTEGRATED: u64;
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
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

#[derive(Serialize, Deserialize)]
pub struct Address<TPrefix: AddressPrefixConfig> {
    #[serde(skip)]
    pub address_type: AddressType,

    pub spend_public_key: PublicKey,
    pub view_public_key: PublicKey,

    marker: std::marker::PhantomData<TPrefix>
}

impl<TPrefix: AddressPrefixConfig> Address<TPrefix> {
    pub fn standard(spend_public_key: PublicKey, view_public_key: PublicKey) -> Self {
        Address {
            address_type: AddressType::Standard,
            spend_public_key,
            view_public_key,
            marker: std::marker::PhantomData
        }
    }

    pub fn subaddress(spend_public_key: PublicKey, view_public_key: PublicKey) -> Self {
        Address {
            address_type: AddressType::SubAddress,
            spend_public_key,
            view_public_key,
            marker: std::marker::PhantomData
        }
    }

    pub fn integrated(spend_public_key: PublicKey, view_public_key: PublicKey, _payment_id: crypto::Hash256) -> Self {
        Address {
            address_type: AddressType::Integrated(),
            spend_public_key,
            view_public_key,
            marker: std::marker::PhantomData
        }
    }
}

/// Get the string representation of an address
impl<TPrefix: AddressPrefixConfig> Into<String> for Address<TPrefix> {
    fn into(self) -> String {
        let mut address = Vec::new();

        // Tag
        let tag = match self.address_type {
            AddressType::Standard     => TPrefix::STANDARD,
            AddressType::SubAddress   => TPrefix::SUBADDRESS,
            AddressType::Integrated() => TPrefix::INTEGRATED
        };
        address.extend_from_slice(&bincode_epee::serialize(&tag).unwrap());

        // Spend and view public keys
        address.extend_from_slice(&bincode_epee::serialize(&self).unwrap());

        // Base58
        base58_monero::encode_check(&address).unwrap()
    }
}

/// Get an Address from its string representation
impl<TPrefix: AddressPrefixConfig> TryFrom<&str> for Address<TPrefix> {
    type Error = failure::Error;

    fn try_from(data: &str) -> Result<Self, Self::Error> {
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

        // HACK: Since bincode_epee is meant to be a one-way encoder to Monero's serialization,
        //       this code just slices the correct portions of the address data to get the spend
        //       and view public keys. Doable for small structs like this one, but for larger
        //       structs we'll need proper deserialization.
        let spend_public_key = PublicKey::from_slice(&data[(tag_end     )..(tag_end + 32)]);
        let view_public_key  = PublicKey::from_slice(&data[(tag_end + 32)..(tag_end + 64)]);

        // TODO: Implement bincode_epee deserialization to fix this monstrosity
        let tag = &data[0..tag_end];

        if tag == bincode_epee::serialize(&TPrefix::STANDARD).unwrap().as_slice() {
            Ok(Address::standard(spend_public_key, view_public_key))
        } else if tag == bincode_epee::serialize(&TPrefix::SUBADDRESS).unwrap().as_slice() {
            Ok(Address::subaddress(spend_public_key, view_public_key))
        } else if tag == bincode_epee::serialize(&TPrefix::INTEGRATED).unwrap().as_slice() {
            Ok(Address::integrated(spend_public_key, view_public_key, crypto::Hash256::null_hash()))
        } else {
            Err(failure::format_err!("Invalid address prefix"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: Deduplicate this for a common setup
    struct TestPrefixes;

    impl AddressPrefixConfig for TestPrefixes {
        const STANDARD:   u64 = 0x0014_5023; // UNP
        const SUBADDRESS: u64 = 0x0021_1023; // UNPS
        const INTEGRATED: u64 = 0x0029_1023; // UNPi
    }

    #[test]
    fn it_encodes_addresses_properly() {
        // Unprll Donation address
        let spend_public_key = PublicKey::from_slice(&hex::decode("1ed50fe76f3fcd23c16493f8802b04f1c77eace5a54f969cc03dfa5cd3149457").unwrap());
        let view_public_key =  PublicKey::from_slice(&hex::decode("36440552e76c9029d22edb4db283b0d9daf2ed21001728248eb4300eaba7f4e0").unwrap());

        let address: Address<TestPrefixes> = Address::standard(spend_public_key, view_public_key);
        let address: String = address.into();

        assert_eq!(
            address,
            String::from("UNP1Yn4gC4EBfxGByWr4CX8CLnvLRm3ZWEK7BEeiuwYe4SeVpqbRMZxKACWXQ1WCw3P2Zpt68rHZ94sehkF5o8Wn7NAC1PoBzh")
        );
    }

    #[test]
    fn it_decodes_standard_string_addresses_properly() {
        // Unprll Donation address
        let address: Address<TestPrefixes> = Address::try_from("UNP1Yn4gC4EBfxGByWr4CX8CLnvLRm3ZWEK7BEeiuwYe4SeVpqbRMZxKACWXQ1WCw3P2Zpt68rHZ94sehkF5o8Wn7NAC1PoBzh").unwrap();

        // Address type
        assert_eq!(
            address.address_type,
            AddressType::Standard
        );

        // Spend public key
        assert_eq!(
            hex::encode(address.spend_public_key.as_bytes()),
            "1ed50fe76f3fcd23c16493f8802b04f1c77eace5a54f969cc03dfa5cd3149457"
        );

        // View public key
        assert_eq!(
            hex::encode(address.view_public_key.as_bytes()),
            "36440552e76c9029d22edb4db283b0d9daf2ed21001728248eb4300eaba7f4e0"
        );
    }

    #[test]
    fn it_decodes_subaddress_string_addresses_properly() {
        // Unprll Donation wallet subaddress
        let address: Address<TestPrefixes> = Address::try_from("UNPStVLMoCzdHGE7EeVNuuWeReeJQXDeEWtRfaCQJ7oJSMr4bYVpreqcP36SjwiCHF86z9bbQecaqcW6yH5ndWx2M6t69dcEoE2").unwrap();

        // Address type
        assert_eq!(
            address.address_type,
            AddressType::SubAddress
        );

        // Spend public key
        assert_eq!(
            hex::encode(address.spend_public_key.as_bytes()),
            "b4c9093fd8e8013eb396e5c0b13cc7819b968c9fb2ae39333c2f078d979d304c"
        );

        // View public key
        assert_eq!(
            hex::encode(address.view_public_key.as_bytes()),
            "be9156eed385d61060ea2d022a779c4b28ecc68ed440517a2a8a0c7b782daa66"
        );
    }
}
