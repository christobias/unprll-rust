#![deny(missing_docs)]

//! Cryptonote Wallet library
//!
//! Used to handle Cryptonote wallets

#[macro_use]
extern crate failure;

use std::collections::HashMap;
use std::convert::From;

use serde::{Deserialize, Serialize};

use transaction_util::{
    AccountKeys,
    address::{Address, AddressPrefixes},
    subaddress::{self, SubAddressIndex},
};
use crypto::{Hash256, SecretKey};

mod account;
mod output_scanning;

#[cfg(test)]
mod test_definitions;

use account::Account;

/// A Cryptonote Wallet
#[derive(Serialize, Deserialize)]
pub struct Wallet<TCoin>
where
    TCoin: AddressPrefixes,
{
    account_keys: AccountKeys,
    accounts: HashMap<u32, Account<TCoin>>,
    checked_blocks: HashMap<u64, Hash256>,
}

/// Generate a wallet instance from an existing AccountKeys struct
impl<TCoin> From<AccountKeys> for Wallet<TCoin>
where
    TCoin: AddressPrefixes,
{
    fn from(account_keys: AccountKeys) -> Self {
        let mut w = Wallet {
            account_keys,
            accounts: HashMap::new(),
            checked_blocks: HashMap::new(),
        };

        // Insert standard address in the accounts map
        w.accounts.insert(
            0,
            Account::new(Address::standard(
                w.account_keys.spend_keypair.public_key,
                w.account_keys.view_keypair.public_key,
            )),
        );

        // Mark genesis as checked
        // FIXME: Probably inefficient
        use common::GetHash;
        w.checked_blocks
            .insert(0, common::Block::genesis().get_hash());

        w
    }
}

impl<TCoin> Wallet<TCoin>
where
    TCoin: AddressPrefixes,
{
    /// Deterministic wallet generation
    ///
    /// This allows having to store only one value (the spend secret key) while the others are computed
    /// The view secret key is derived by taking the Keccak (non-standard) hash of the spend secret key
    pub fn from_spend_secret_key(spend_secret_key: SecretKey) -> Self {
        let account_keys = AccountKeys::from(spend_secret_key);

        Self::from(account_keys)
    }

    /// Shortcut method for determining the address for the given subaddress index
    pub fn get_address_for_index(&self, index: &SubAddressIndex) -> Address<TCoin> {
        subaddress::get_address_for_index(&self.account_keys, &index)
    }
}

#[cfg(test)]
mod tests {
    use crypto::ScalarExt;

    use super::*;
    use test_definitions::TestCoin;

    #[test]
    fn it_works() {
        let w: Wallet<TestCoin> = Wallet::from_spend_secret_key(SecretKey::from_slice(
            &hex::decode("91ca5959117826861a8d3dba04ef036aba07ca4e02b9acf28fc1e3af25c4400a")
                .unwrap(),
        ));

        // This given set of keys is that of a testnet wallet. As all keys are in public view,
        // DO NOT use this wallet for storing any coins

        // Spend private key (pedantic check)
        assert_eq!(
            hex::encode(w.account_keys.spend_keypair.secret_key.to_bytes()),
            "91ca5959117826861a8d3dba04ef036aba07ca4e02b9acf28fc1e3af25c4400a"
        );
        // Spend public key
        assert_eq!(
            hex::encode(w.account_keys.spend_keypair.public_key.to_bytes()),
            "4dcff6ae0b5313938e718bb033907fee6cddc053f4d44c41bd0f9fed5ea7cef7"
        );

        // View secret key
        assert_eq!(
            hex::encode(w.account_keys.view_keypair.secret_key.to_bytes()),
            "84bc8a0314bfa06dee4b992cca4420d19f28af37f4fb90e031454c66f8cd6003"
        );

        // View public key
        assert_eq!(
            hex::encode(w.account_keys.view_keypair.public_key.to_bytes()),
            "8b66a0e272063786cc769c295486552e39797c57243612047bff9845c8cc66c8"
        );
    }
}
