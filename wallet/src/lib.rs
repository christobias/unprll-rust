use std::collections::HashMap;
use std::convert::From;

use serde::{
    Serialize,
    Deserialize
};

use crypto::{
    CNFastHash,
    Digest,
    KeyPair,
    ScalarExt,
    SecretKey
};

pub mod address;
mod block_scanning_ops;
mod subaddress_ops;

#[cfg(test)]
mod test_definitions;

use address::Address;

#[derive(Debug, Eq, Clone, Hash, PartialEq, Serialize, Deserialize)]
pub struct SubAddressIndex(pub u32, pub u32);

#[derive(Serialize, Deserialize)]
pub struct Wallet<TCoinConfig>
where
    TCoinConfig: address::AddressPrefixConfig
{
    spend_keypair: KeyPair,
    view_keypair: KeyPair,

    addresses: HashMap<SubAddressIndex, Address<TCoinConfig>>
}

impl<TCoinConfig> Wallet<TCoinConfig>
where
    TCoinConfig: address::AddressPrefixConfig
{
    /// Generate a wallet instance from a spend secret key and view secret key
    pub fn from_secret_keys(spend_secret_key: SecretKey, view_secret_key: SecretKey) -> Self {
        let mut w = Wallet {
            spend_keypair: KeyPair::from(spend_secret_key),
            view_keypair: KeyPair::from(view_secret_key),
            addresses: HashMap::new()
        };

        // Insert standard address in the subaddresses map
        w.addresses.insert(SubAddressIndex(0, 0), Address::standard(w.spend_keypair.public_key, w.view_keypair.public_key));

        w
    }

    /// Deterministic wallet generation
    ///
    /// This allows having to store only one value (the spend secret key) while the others are computed
    /// The view secret key is derived by taking the Keccak (non-standard) hash of the spend secret key
    pub fn from(spend_secret_key: SecretKey) -> Self {
        let view_secret_key = SecretKey::from_slice(&CNFastHash::digest(&spend_secret_key.to_bytes()));

        Self::from_secret_keys(spend_secret_key, view_secret_key)
    }

    pub fn get_address_for_index(&self, index: &SubAddressIndex) -> Option<&Address<TCoinConfig>> {
        self.addresses.get(index)
    }

    pub fn spend_keypair(&self) -> &KeyPair {
        &self.spend_keypair
    }
    pub fn view_keypair(&self) -> &KeyPair {
        &self.view_keypair
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_definitions::TestCoin;

    #[test]
    fn it_works() {
        let w: Wallet<TestCoin> = Wallet::from(SecretKey::from_slice(&hex::decode("91ca5959117826861a8d3dba04ef036aba07ca4e02b9acf28fc1e3af25c4400a").unwrap()));

        // This given set of keys is that of a testnet wallet. As all keys are in public view,
        // DO NOT use this wallet for storing any coins

        // Spend private key (pedantic check)
        assert_eq!(
            hex::encode(w.spend_keypair().secret_key.to_bytes()),
            "91ca5959117826861a8d3dba04ef036aba07ca4e02b9acf28fc1e3af25c4400a"
        );
        // Spend public key
        assert_eq!(
            hex::encode(w.spend_keypair().public_key.to_bytes()),
            "4dcff6ae0b5313938e718bb033907fee6cddc053f4d44c41bd0f9fed5ea7cef7"
        );

        // View secret key
        assert_eq!(
            hex::encode(w.view_keypair().secret_key.to_bytes()),
            "84bc8a0314bfa06dee4b992cca4420d19f28af37f4fb90e031454c66f8cd6003"
        );

        // View public key
        assert_eq!(
            hex::encode(w.view_keypair().public_key.to_bytes()),
            "8b66a0e272063786cc769c295486552e39797c57243612047bff9845c8cc66c8"
        );
    }
}
