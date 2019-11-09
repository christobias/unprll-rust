use std::collections::HashMap;
use std::convert::From;

use serde::{
    Serialize,
    Deserialize
};

use crypto::{
    CNFastHash,
    Digest,
    Hash256,
    KeyPair,
    ScalarExt,
    SecretKey
};

mod account;
mod address;
mod output_scanning;

#[cfg(test)]
mod test_definitions;

use account::Account;
pub use address::{
    Address,
    AddressPrefixes,
    SubAddressIndex
};

#[derive(Serialize, Deserialize)]
pub struct Wallet<TCoin>
where
    TCoin: address::AddressPrefixes
{
    spend_keypair: KeyPair,
    view_keypair: KeyPair,

    accounts: HashMap<u32, Account<TCoin>>,

    checked_blocks: HashMap<u64, Hash256>
}

impl<TCoin> Wallet<TCoin>
where
    TCoin: address::AddressPrefixes
{
    /// Generate a wallet instance from a spend secret key and view secret key
    pub fn from_secret_keys(spend_secret_key: SecretKey, view_secret_key: SecretKey) -> Self {
        let mut w = Wallet {
            spend_keypair: KeyPair::from(spend_secret_key),
            view_keypair: KeyPair::from(view_secret_key),
            accounts: HashMap::new(),
            checked_blocks: HashMap::new()
        };

        // Insert standard address in the accounts map
        w.accounts.insert(0, Account::new(Address::standard(w.spend_keypair.public_key, w.view_keypair.public_key)));

        // Mark genesis as checked
        // FIXME: Probably inefficient
        use common::GetHash;
        w.checked_blocks.insert(0, common::Block::genesis().get_hash());

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

    pub fn spend_keypair(&self) -> &KeyPair {
        &self.spend_keypair
    }
    pub fn view_keypair(&self) -> &KeyPair {
        &self.view_keypair
    }
    pub fn accounts(&self) -> &HashMap<u32, Account<TCoin>> {
        &self.accounts
    }
    pub fn checked_blocks(&self) -> &HashMap<u64, Hash256> {
        &self.checked_blocks
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
