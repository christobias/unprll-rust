use std::collections::HashMap;

use byteorder::{ByteOrder, LittleEndian};
use serde::{Deserialize, Serialize};

use crypto::{
    ecc::CompressedPoint,
    Hash256, Hash8, KeyImage,
};
use ringct::Commitment;
use transaction_util::subaddress::{SubAddressIndex};

use crate::{Wallet};

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnspentOutput {
    pub commitment: Commitment,
    pub block_height: u64,
    pub minor_index: u32,
    pub payment_id: Option<Hash8>,
    pub txid: Hash256,
}

#[derive(Serialize, Deserialize)]
pub struct Account {
    subaddress_indices: Vec<u32>,
    unspent_outputs: HashMap<CompressedPoint, UnspentOutput>
}

impl Account {
    pub fn subaddress_indices(&self) -> &Vec<u32> {
        &self.subaddress_indices
    }
    pub fn get_balance(&self) -> u64 {
        self.unspent_outputs.iter().fold(0u64, |acc, (_, curr)| {
            acc + LittleEndian::read_u64(curr.commitment.value.as_bytes())
        })
    }
    pub fn add_unspent_output(&mut self, key_image: KeyImage, output: UnspentOutput) {
        if self.unspent_outputs.contains_key(&key_image.compress()) {
            // TODO: Is panicking reasonable? This situation must never occur,
            //       otherwise we'll have multiple outputs with the same key image
            unreachable!()
        }
        self.unspent_outputs.insert(key_image.compress(), output);
    }
    pub fn mark_output_as_spent(&mut self, key_image: KeyImage) {
        self.unspent_outputs.remove(&key_image.compress());
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            subaddress_indices: vec!{0},
            unspent_outputs: HashMap::new(),
        }
    }
}

impl Wallet {
    /// Add an account to the current wallet
    pub fn add_account(&mut self, major_index: u32) {
        self.accounts.insert(
            major_index,
            Account::default(),
        );
    }
    /// Get the account at the given major index from the current wallet
    pub fn get_account(&self, major_index: u32) -> Option<&Account> {
        self.accounts.get(&major_index)
    }

    /// Add an address to the given account
    pub fn add_address(&mut self, index: SubAddressIndex) -> Option<()> {
        let account = self.accounts.get_mut(&index.0)?;

        account.subaddress_indices.push(index.1);
        Some(())
    }
}
