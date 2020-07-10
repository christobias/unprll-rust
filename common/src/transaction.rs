use digest::Digest;
use serde::{Deserialize, Serialize};

use crypto::{CNFastHash, Hash256, Hash256Data, Hash8, KeyImage, PublicKey};
use ringct::RingCTSignature;

use crate::GetHash;

/// Transaction input
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TXIn {
    /// Coinbase (genesis) input. Creates new coins
    /// Contains the Block height of this transaction
    Gen(u64),
    /// Coins from an existing "ToKey" output
    FromKey {
        /// Relative offsets of each output in the ring
        key_offsets: Vec<u64>,
        /// Key image of the sender's output
        key_image: KeyImage,
    },
}

/// Transaction output target
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TXOutTarget {
    /// Send to specified public key
    ToKey {
        /// Target public key
        key: PublicKey,
    },
}

/// Transaction output
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct TXOut {
    /// Amount of coins received (0 for RingCT)
    pub amount: u64,
    /// Transaction output target
    pub target: TXOutTarget,
}

/// Nonces added to the transaction
///
/// Currently stores only the encrypted payment ID
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TXNonce {
    /// The Encrypted Payment ID for this transaction
    ///
    /// Useful for identifying transactions by the receiver
    EncryptedPaymentId(Hash8),
}

/// Extra information added to the transaction
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum TXExtra {
    /// Public key of this transaction (for determining output secret key)
    TxPublicKey(PublicKey),
    /// Additional public keys for this transaction
    TxAdditionalPublicKeys(Vec<PublicKey>),
    /// Nonces for this transaction
    TxNonce(TXNonce),
}

/// Transaction prefix
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct TransactionPrefix {
    /// This transaction's version
    pub version: usize,
    /// How many block "deltas" this block is locked for
    pub unlock_delta: u16,
    /// List of inputs to this transaction
    pub inputs: Vec<TXIn>,
    /// List of outputs in this transaction
    pub outputs: Vec<TXOut>,
    /// Extra information tagged to this transaction
    pub extra: Vec<TXExtra>,
}

/// A complete Transaction
#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub struct Transaction {
    /// This transaction's prefix
    pub prefix: TransactionPrefix,

    /// RingCT Signatures to prove ownership, authorize the transaction and hide amounts
    pub rct_signatures: Vec<RingCTSignature>,
}

impl GetHash for TransactionPrefix {
    fn get_hash_blob(&self) -> Vec<u8> {
        let mut vec = Vec::new();

        // Tx version
        vec.extend_from_slice(&varint::serialize(self.version as u64));

        // Unlock delta
        vec.extend_from_slice(&varint::serialize(self.unlock_delta as u64));

        // Inputs
        vec.extend_from_slice(&varint::serialize(self.inputs.len() as u64));
        for input in &self.inputs {
            match input {
                TXIn::Gen(height) => {
                    // Enum tag
                    vec.push(0xff);

                    // Input
                    vec.extend_from_slice(&varint::serialize(*height as u64));
                }
                _ => unimplemented!(),
            }
        }

        // Outputs
        vec.extend_from_slice(&varint::serialize(self.outputs.len() as u64));
        for output in &self.outputs {
            // Amount
            vec.extend_from_slice(&varint::serialize(output.amount as u64));

            // Target
            match output.target {
                TXOutTarget::ToKey { key } => {
                    // Enum tag
                    vec.extend_from_slice(&varint::serialize(0x02 as u64));

                    // Public Key
                    vec.extend_from_slice(key.compress().as_bytes());
                }
            }
        }

        // Extra
        let mut extra_buf = Vec::new();
        for extra in &self.extra {
            match extra {
                TXExtra::TxPublicKey(key) => {
                    // Enum tag
                    extra_buf.extend_from_slice(&varint::serialize(0x01 as u64));

                    // Public Key
                    extra_buf.extend_from_slice(key.compress().as_bytes());
                }
                TXExtra::TxAdditionalPublicKeys(keys) => {
                    // Enum tag
                    extra_buf.extend_from_slice(&varint::serialize(0x04 as u64));
                    // Public Keys
                    for key in keys {
                        extra_buf.extend_from_slice(key.compress().as_bytes());
                    }
                }
                TXExtra::TxNonce(nonce) => {
                    // Enum tag
                    extra_buf.extend_from_slice(&varint::serialize(0x02 as u64));

                    match nonce {
                        TXNonce::EncryptedPaymentId(payment_id) => {
                            // Enum tag
                            extra_buf.extend_from_slice(&varint::serialize(0x01 as u64));

                            // Payment ID
                            extra_buf.extend_from_slice(payment_id.data());
                        }
                    }
                }
            }
        }
        vec.extend_from_slice(&varint::serialize(extra_buf.len() as u64));
        vec.extend_from_slice(&extra_buf);
        vec
    }
}

impl GetHash for Transaction {
    fn get_hash_blob(&self) -> Vec<u8> {
        let mut vec = self.prefix.get_hash_blob();
        vec
    }
    fn get_hash(&self) -> Hash256 {
        if self.prefix.version == 1 {
            return Hash256::from(CNFastHash::digest(&self.get_hash_blob()));
        }
        let mut hashes: Vec<Hash256Data> = Vec::with_capacity(3);
        // Prefix hash
        hashes.push(CNFastHash::digest(
            &bincode::serialize(&self.prefix).unwrap(),
        ));
        // Signatures hash
        // TODO: Put the empty hash needed for hash-compatibility
        // hashes.push(CNFastHash::digest(
        //     &bincode::serialize(&self.signatures).unwrap(),
        // ));
        // TODO: RingCT Signatures hash
        // hashes[2] = CNFastHash::digest(&bincode::serialize(&self.signatures).unwrap());

        Hash256::from(CNFastHash::digest(&bincode::serialize(&hashes).unwrap()))
    }
}
