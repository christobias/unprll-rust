use std::collections::HashMap;

use common::{GetHash, PreliminaryChecks, Transaction, TXIn, TXExtra, TXNonce};
use crypto::Hash256;
use ensure_macro::ensure;
use ringct::{Error as RingCTError};

type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Wrong transaction version. Expected {}", expected)]
    WrongTransactionVersion { expected: u16 },

    #[error("Incorrect payment ID count")]
    IncorrectPaymentIDCount,

    #[error("Invalid transaction input")]
    InvalidTransactionInput,

    #[error(transparent)]
    RingCT(#[from] RingCTError),
}

/// A memory pool of unconfirmed transactions
/// 
/// Handles transaction verification as transactions can only
/// be confirmed if they are in the transaction pool
#[derive(Default)]
pub struct TXPool {
    transactions: HashMap<Hash256, Transaction>,
}

impl TXPool {
    /// Creates a new TXPool
    pub fn new() -> Self {
        TXPool {
            transactions: HashMap::new(),
        }
    }

    /// Add an unconfirmed transaction to the TXPool
    pub fn add_transactions(&mut self, transactions: &[Transaction]) -> Result<()> {
        self.check(&transactions)?;

        for tx in transactions {
            self.transactions.insert(tx.get_hash(), tx.clone());
        }

        Ok(())
    }

    /// Check if this TXPool contains the given transaction using the txid
    pub fn has_transaction(&self, txid: &Hash256) -> bool {
        self.transactions.contains_key(txid)
    }

    /// Takes the transaction, removing it from the TXPool in the process
    pub fn take_transaction(&mut self, txid: &Hash256) -> Option<Transaction> {
        self.transactions.remove(txid)
    }
}

impl PreliminaryChecks<&[Transaction]> for TXPool {
    type Error = Error;

    fn check(&self, transactions: &&[Transaction]) -> Result<()> {
        for tx in transactions.iter() {
            // All transactions must be v2 (RingCT enabled)
            ensure!(tx.prefix.version == 2, Error::WrongTransactionVersion { expected: 2 });

            // Find all payment IDs (should be just one)
            let payment_ids = tx.prefix.extra.iter()
                .filter_map(|extra| {
                    if let TXExtra::TxNonce(TXNonce::EncryptedPaymentId(payment_id)) = extra {
                        Some(payment_id)
                    } else {
                        None
                    }
                }).collect::<Vec<_>>();

            // All transactions must have a single encrypted payment ID
            ensure!(payment_ids.len() == 1, Error::IncorrectPaymentIDCount);

            for input in &tx.prefix.inputs {
                // All inputs must be TXIn::FromKey (TXIn::Gen is from miner transactions only)
                match input {
                    TXIn::Gen(_) => return Err(Error::InvalidTransactionInput),
                    TXIn::FromKey { .. } => {}
                }
            }

            // TODO: Check transaction fee
            // TODO: Check transaction weight
        }

        let signatures = transactions
            .iter()
            .map(|tx| tx.rct_signature.as_ref().unwrap())
            .collect::<Vec<_>>();

        ringct::verify_multiple(&signatures)?;

        Ok(())
    }
}
