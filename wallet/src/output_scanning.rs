use std::collections::HashMap;

use log::info;

use common::{Block, GetHash, TXExtra, TXOutTarget, Transaction};
use crypto::{CNFastHash, Digest, Hash256, SecretKey};

use crate::{AddressPrefixes, SubAddressIndex, Wallet};

impl<TCoin> Wallet<TCoin>
where
    TCoin: AddressPrefixes,
{
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

    fn scan_transaction(&mut self, transaction: &Transaction) -> Option<SecretKey> {
        for output in &transaction.prefix.outputs {
            let mut tx_pub_key = None;
            for extra in &transaction.prefix.extra {
                match extra {
                    TXExtra::TxPublicKey(key) => {
                        tx_pub_key = Some(key.decompress().unwrap());
                    }
                }
            }

            if let Some(tx_pub_key) = tx_pub_key {
                match output.target {
                    TXOutTarget::ToKey { key } => {
                        // Compute the common "tx scalar"
                        // H_s(aR)
                        let tx_scalar = crypto::ecc::hash_to_scalar(CNFastHash::digest(
                            (self.view_keypair.secret_key * tx_pub_key)
                                .compress()
                                .as_bytes(),
                        ));

                        // Do the original Cryptonote derivation first
                        // H_s(aR)G + B
                        let computed_pub_key = &tx_scalar * &crypto::ecc::BASEPOINT_TABLE
                            + self.spend_keypair.public_key.decompress().unwrap();

                        // Check if the output is to our standard address
                        let index_address_pair = if tx_pub_key == computed_pub_key {
                            // It's to our standard address
                            Some((
                                SubAddressIndex(0, 0),
                                self.accounts.get(&0).unwrap().addresses().get(&0).unwrap(),
                            ))
                        } else {
                            // Try the subaddress derivation next
                            // P - H_s(aR)G
                            let computed_pub_key = key.decompress().unwrap()
                                - &tx_scalar * &crypto::ecc::BASEPOINT_TABLE;
                            let computed_pub_key = computed_pub_key.compress();

                            // Find the corresponding public spend key
                            self.accounts
                                .iter()
                                .flat_map(|(major, account)| {
                                    account.addresses().iter().map(move |(minor, address)| {
                                        (SubAddressIndex(*major, *minor), address)
                                    })
                                })
                                .find(|(_, address)| address.spend_public_key == computed_pub_key)
                        };

                        if let Some((index, _address)) = index_address_pair {
                            info!("Output found in txid <{}>", transaction.get_hash());
                            let output_secret_key = if index == SubAddressIndex(0, 0) {
                                // Main address derives things differently
                                // H_s(aR) + b
                                tx_scalar + self.spend_keypair.secret_key
                            } else {
                                // H_s(aR) + b + m_i
                                tx_scalar
                                    + self.spend_keypair.secret_key
                                    + self.get_subaddress_secret_key(&index)
                            };

                            self.accounts
                                .get_mut(&index.0)
                                .unwrap()
                                .increment_balance(output.amount);

                            return Some(output_secret_key);
                        }
                    }
                }
            };
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use common::{TXOut, TransactionPrefix};
    use crypto::KeyPair;

    use crate::address::AddressType;
    use crate::test_definitions::TestCoin;

    use super::*;

    #[test]
    fn it_receives_outputs_correctly() {
        // A test wallet
        let mut wallet: Wallet<TestCoin> = Wallet::from(KeyPair::generate().secret_key);

        // TODO: Replace with actual transaction sending code
        [
            // The standard address recognizes inputs differently
            (0, 0),
            // Some subaddresses
            (1, 0),
            (1, 1),
            (100, 100),
            (32767, 256),
        ]
        .iter()
        // Convert to SubAddressIndex
        .map(|(major, minor)| SubAddressIndex(*major, *minor))
        .for_each(|index| {
            // Add the subaddress and get its public keys
            wallet.add_account(index.0);
            wallet.add_address(index.clone()).unwrap();
            let address = wallet.get_address_for_index(&index);

            // r
            let random_scalar = KeyPair::generate().secret_key;

            let tx_pub_key = if let AddressType::Standard = address.address_type {
                // rG
                &random_scalar * &crypto::ecc::BASEPOINT_TABLE
            } else {
                // rD
                random_scalar * address.spend_public_key.decompress().unwrap()
            };

            // H_s(rC)
            let tx_scalar = crypto::ecc::hash_to_scalar(CNFastHash::digest(
                (random_scalar * address.view_public_key.decompress().unwrap())
                    .compress()
                    .as_bytes(),
            ));

            // H_s(rC)*G + D
            let tx_dest_key = &tx_scalar * &crypto::ecc::BASEPOINT_TABLE
                + address.spend_public_key.decompress().unwrap();

            let t = Transaction {
                prefix: TransactionPrefix {
                    version: 1,
                    unlock_delta: 0,
                    inputs: Vec::default(),
                    outputs: vec![TXOut {
                        amount: 0,
                        target: TXOutTarget::ToKey {
                            key: tx_dest_key.compress(),
                        },
                    }],
                    extra: vec![TXExtra::TxPublicKey(tx_pub_key.compress())],
                },
                signatures: Vec::new(),
            };

            // Scan the transaction
            let tx_scan_result = wallet.scan_transaction(&t);

            // The tx output must be detected
            assert!(tx_scan_result.is_some());

            // The tx secret key must correspond to the tx destination key
            assert!(&tx_scan_result.unwrap() * &crypto::ecc::BASEPOINT_TABLE == tx_dest_key);
        });
    }
}
