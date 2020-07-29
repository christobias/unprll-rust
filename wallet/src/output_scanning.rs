use std::collections::HashMap;

use common::{Block, GetHash, TXExtra, TXIn, TXNonce, TXOutTarget, Transaction};
use crypto::{Hash256, Hash8, KeyImage, PublicKey, SecretKey};
use ringct::Commitment;
use transaction_util::{payment_id, tx_scanning, Derivation};

use crate::{account::UnspentOutput, SubAddressIndex, Wallet};

struct TXScanInfo {
    commitment: Commitment,
    key_image: KeyImage,
    output_public_key: PublicKey,
    subaddress_index: SubAddressIndex,
    payment_id: Option<Hash8>,
}

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

            // TODO: Remove unspent outputs from blocks above split
        }

        // Get the block height
        let block_height = if let TXIn::Gen(height) = block.miner_tx.prefix.inputs[0] {
            height
        } else {
            unreachable!();
        };

        // Scan the coinbase transaction first
        // Assumes the coinbase transaction only contains one output
        let miner_tx_hash = block.miner_tx.get_hash();
        let mut tx_scans = vec![(&miner_tx_hash, self.scan_transaction(&block.miner_tx))];

        // Then scan each transaction in the block
        for txid in &block.tx_hashes {
            // TODO: Handle missing transactions
            tx_scans.push((txid, self.scan_transaction(transactions.get(txid).unwrap())));
        }

        for (txid, tx_scans_vec) in tx_scans {
            for tx_scan_info in tx_scans_vec {
                // We've got money!
                log::info!(
                    "Output found in txid <{}>. Output public key <{}>",
                    txid,
                    hex::encode(tx_scan_info.output_public_key.compress().as_bytes())
                );

                // Add the output's amount to the corresponding account
                self.accounts
                    .get_mut(&tx_scan_info.subaddress_index.0)
                    .unwrap()
                    .add_unspent_output(
                        tx_scan_info.key_image,
                        UnspentOutput {
                            commitment: tx_scan_info.commitment,
                            block_height,
                            minor_index: tx_scan_info.subaddress_index.1,
                            payment_id: tx_scan_info.payment_id.clone(),
                            txid: txid.clone(),
                        },
                    );
            }
        }

        // Add this block to the list of scanned blocks
        // TODO: There's probably a more efficient way
        if !self.checked_blocks.is_empty() {
            let (&current_height, _) = self.get_last_checked_block();
            self.checked_blocks.insert(current_height + 1, block_id);
        }
    }

    fn scan_transaction(&self, transaction: &Transaction) -> Vec<TXScanInfo> {
        // Grab the transaction public keys and the payment ID
        let mut tx_pub_keys = Vec::new();
        let mut payment_id = None;

        for extra in &transaction.prefix.extra {
            match extra {
                TXExtra::TxPublicKey(key) => {
                    tx_pub_keys.push(*key);
                }
                TXExtra::TxAdditionalPublicKeys(keys) => {
                    tx_pub_keys.extend_from_slice(keys);
                }
                TXExtra::TxNonce(nonce) => {
                    match nonce {
                        TXNonce::EncryptedPaymentId(encrypted_payment_id) => {
                            // TODO: Think about multi-payment ID scenarios
                            payment_id = Some(payment_id::decrypt(
                                encrypted_payment_id,
                                Derivation::from(
                                    &self.account_keys.view_keypair.secret_key,
                                    &tx_pub_keys[0],
                                )
                                .unwrap(),
                            ));
                        }
                    }
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
                    .subaddress_indices()
                    .iter()
                    .map(move |minor| SubAddressIndex(*major, *minor))
            })
            .collect::<Vec<_>>();

        // Iterator-based to allow easier parallelization later
        transaction
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
                    let key_image = tx_scanning::get_key_image(
                        &self.account_keys,
                        sub_index,
                        &output_public_key,
                        output_index as u64,
                        &tx_pub_keys,
                    );

                    if let Some((key_image, ephemeral_keypair)) = key_image {
                        let commitment = if let Some(rct_signature) = &transaction.rct_signature {
                            // TODO: Handle errors
                            ringct::decode(
                                rct_signature,
                                output_index,
                                &ephemeral_keypair.secret_key,
                            )
                            .unwrap()
                        } else {
                            Commitment {
                                value: SecretKey::from(output.amount),
                                mask: SecretKey::one(),
                            }
                        };

                        return Some(TXScanInfo {
                            commitment,
                            key_image,
                            output_public_key,
                            subaddress_index: sub_index.clone(),
                            payment_id: payment_id.clone(),
                        });
                    }
                }

                None
            })
            .collect()
    }
}
