use std::collections::HashMap;

use log::info;

use common::{Block, GetHash, TXExtra, TXOutTarget, Transaction};
use crypto::{Hash256, SecretKey};
use transaction_util::tx_scanning;

use crate::{SubAddressIndex, Wallet};

impl Wallet {
    /// Get the last checked block of the current wallet
    pub fn get_last_checked_block(&self) -> (&u64, &Hash256) {
        self.checked_blocks
            .iter()
            .max_by(|(height_1, _), (height_2, _)| height_1.cmp(height_2))
            .unwrap()
    }
    /// Scan a given block for transactions to the current wallet
    ///
    /// First scans the coinbase transaction, then all other transactions in the block
    pub fn scan_block(&mut self, block: &Block, transactions: &HashMap<Hash256, Transaction>) {
        // Check if we're scanning an older block height, in which case
        // we'll need to rescan from that point (possibly due to a reorg)
        let block_id = block.get_hash();
        if let Some((&split_height, _)) =
            self.checked_blocks.iter().find(|(_, id)| id == &&block_id)
        {
            // Remove all blocks at and above the split point
            self.checked_blocks
                .retain(|&height, _| height <= split_height);

            // TODO: Remove output keys from blocks above split
        }

        // Scan the coinbase transaction first
        self.scan_transaction(&block.miner_tx);

        // Then scan each transaction in the block
        for txid in &block.tx_hashes {
            // TODO: Handle missing transactions
            self.scan_transaction(transactions.get(txid).unwrap());
        }

        // Add this block to the list of scanned blocks
        // TODO: There's probably a more efficient way
        if !self.checked_blocks.is_empty() {
            let (&current_height, _) = self.get_last_checked_block();
            self.checked_blocks.insert(current_height + 1, block_id);
        }
    }

    fn scan_transaction(&mut self, transaction: &Transaction) -> Option<Vec<SecretKey>> {
        // Grab the transaction public keys
        let mut tx_pub_keys = Vec::new();
        for extra in &transaction.prefix.extra {
            match extra {
                TXExtra::TxPublicKey(key) => {
                    tx_pub_keys.push(*key);
                }
                TXExtra::TxAdditionalPublicKeys(keys) => {
                    tx_pub_keys.extend_from_slice(keys);
                }
                TXExtra::TxNonce(nonce) => {
                    // TODO: Handle payment IDs
                }
            }
        }

        // Keep a copy of each active subaddress
        // TODO: This is inefficient
        let subaddresses = self
            .accounts
            .iter()
            .flat_map(|(major, account)| {
                account
                    .addresses()
                    .iter()
                    .map(move |(minor, _)| SubAddressIndex(*major, *minor))
            })
            .collect::<Vec<_>>();

        let output_secret_keys = transaction
            .prefix
            .outputs
            .iter()
            .enumerate()
            // Filter for outputs that are towards our wallet
            .filter_map(|(output_index, output)| {
                let TXOutTarget::ToKey {
                    key: output_public_key,
                } = output.target;

                for sub_index in &subaddresses {
                    if let Some(output_secret_key) = tx_scanning::get_output_secret_key(
                        &self.account_keys,
                        sub_index,
                        output_index as u64,
                        output_public_key,
                        &tx_pub_keys,
                    ) {
                        // We've got money!
                        info!(
                            "Output found in txid <{}>. Output public key <{}>",
                            transaction.get_hash(),
                            hex::encode(output_public_key.compress().as_bytes())
                        );

                        // Add the output's amount to the corresponding account
                        self.accounts
                            .get_mut(&sub_index.0)
                            .unwrap()
                            .increment_balance(output.amount);

                        return Some(output_secret_key);
                    }
                }

                None
            })
            .collect::<Vec<_>>();

        if output_secret_keys.is_empty() {
            None
        } else {
            Some(output_secret_keys)
        }
    }
}
