use std::collections::HashMap;

use failure::format_err;
use serde::{Deserialize, Serialize};

use crate::{Address, AddressPrefixes, SubAddressIndex, Wallet};

#[derive(Serialize, Deserialize)]
pub struct Account<TCoin>
where
    TCoin: AddressPrefixes,
{
    addresses: HashMap<u32, Address<TCoin>>,
    balance: u64,
}

impl<TCoin> Account<TCoin>
where
    TCoin: AddressPrefixes,
{
    pub fn new(address: Address<TCoin>) -> Self {
        let mut acc = Account {
            addresses: HashMap::new(),
            balance: 0,
        };

        acc.addresses.insert(0, address);

        acc
    }
    pub fn addresses(&self) -> &HashMap<u32, Address<TCoin>> {
        &self.addresses
    }
    pub fn balance(&self) -> u64 {
        self.balance
    }
    pub fn increment_balance(&mut self, inc: u64) -> Result<(), failure::Error> {
        self.balance = self
            .balance
            .checked_add(inc)
            .ok_or_else(|| failure::format_err!("Balance overflow imminent"))?;
        Ok(())
    }
    pub fn decrement_balance(&mut self, inc: u64) -> Result<(), failure::Error> {
        self.balance = self
            .balance
            .checked_sub(inc)
            .ok_or_else(|| failure::format_err!("Balance overflow imminent"))?;
        Ok(())
    }
}

impl<TCoin> Wallet<TCoin>
where
    TCoin: AddressPrefixes,
{
    /// Add an account to the current wallet
    pub fn add_account(&mut self, major_index: u32) {
        self.accounts.insert(
            major_index,
            Account::new(self.get_address_for_index(&SubAddressIndex(major_index, 0))),
        );
    }
    /// Get the account at the given major index from the current wallet
    pub fn get_account(&self, major_index: u32) -> Option<&Account<TCoin>> {
        self.accounts.get(&major_index)
    }

    /// Add an address to the given account
    pub fn add_address(&mut self, index: SubAddressIndex) -> Result<(), failure::Error> {
        let address = self.get_address_for_index(&index);

        let account = self
            .accounts
            .get_mut(&index.0)
            .ok_or_else(|| format_err!("Account at major index {} does not exist!", index.0))?;

        account.addresses.insert(index.1, address);
        Ok(())
    }
}
