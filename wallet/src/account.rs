use std::collections::HashMap;

use failure::{
    format_err
};
use serde::{
    Serialize,
    Deserialize
};

use crate::{
    Address,
    AddressPrefixes,
    SubAddressIndex,
    Wallet
};

#[derive(Serialize, Deserialize)]
pub struct Account<TCoin>
where
    TCoin: AddressPrefixes
{
    addresses: HashMap<u32, Address<TCoin>>,
    balance: u64
}

impl<TCoin> Account<TCoin>
where
    TCoin: AddressPrefixes
{
    pub fn new(address: Address<TCoin>) -> Self {
        let mut acc = Account {
            addresses: HashMap::new(),
            balance: 0
        };

        acc.addresses.insert(0, address);

        acc
    }
    pub fn addresses(&self) -> &HashMap<u32, Address<TCoin>> {
        &self.addresses
    }
}

impl<TCoin> Wallet<TCoin>
where
    TCoin: AddressPrefixes
{
    pub fn add_account(&mut self, major_index: u32) {
        self.accounts.insert(
            major_index,
            Account::new(self.get_address_for_index(&SubAddressIndex(major_index, 0)))
        );
    }
    pub fn get_account(&self, major_index: u32) -> Option<&Account<TCoin>> {
        self.accounts.get(&major_index)
    }

    pub fn add_address(&mut self, index: SubAddressIndex) -> Result<(), failure::Error> {
        let address = self.get_address_for_index(&index);

        let account = self.accounts.get_mut(&index.0)
            .ok_or_else(|| format_err!("Account at major index {} does not exist!", index.0))?;

        account.addresses.insert(index.1, address);
        Ok(())
    }
}
