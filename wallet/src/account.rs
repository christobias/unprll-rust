use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Address, AddressPrefixes, SubAddressIndex, Wallet};

/// Error type for Address operations
#[derive(Fail, Debug)]
pub enum Error {
    /// Returned when the account does not exist at the given index
    #[fail(display = "Account does not exist")]
    DoesNotExist,
}

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
    pub fn increment_balance(&mut self, inc: u64) {
        // TODO: Panicking is probably the more sane alternative
        self.balance = self
            .balance
            .checked_add(inc)
            .expect("Account balance overflow");
    }
    pub fn decrement_balance(&mut self, inc: u64) {
        // TODO: Panicking is probably the more sane alternative
        self.balance = self
            .balance
            .checked_sub(inc)
            .expect("Account balance underflow");
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
    pub fn add_address(&mut self, index: SubAddressIndex) -> Result<(), Error> {
        let address = self.get_address_for_index(&index);

        let account = self.accounts.get_mut(&index.0).ok_or(Error::DoesNotExist)?;

        account.addresses.insert(index.1, address);
        Ok(())
    }
}
