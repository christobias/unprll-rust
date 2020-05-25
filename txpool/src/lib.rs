use std::collections::HashMap;

use failure::Error;

use common::{GetHash, PreliminaryChecks, Transaction};
use crypto::Hash256;

type Result<T> = std::result::Result<T, Error>;

#[derive(Default)]
pub struct TXPool {
    transactions: HashMap<Hash256, Transaction>,
}

impl TXPool {
    pub fn new() -> Self {
        TXPool {
            transactions: HashMap::new(),
        }
    }

    pub fn add_transactions(&mut self, transactions: &[Transaction]) -> Result<()> {
        self.check(&transactions)?;

        for tx in transactions {
            self.transactions.insert(tx.get_hash(), tx.clone());
        }

        Ok(())
    }

    pub fn has_transaction(&self, txid: &Hash256) -> bool {
        self.transactions.contains_key(txid)
    }

    pub fn remove_transaction(&mut self, txid: &Hash256) -> Option<Transaction> {
        self.transactions.remove(txid)
    }
}

impl PreliminaryChecks<&[Transaction]> for TXPool {
    type Error = failure::Error;

    fn check(&self, transactions: &&[Transaction]) -> Result<()> {
        let signatures = transactions
            .iter()
            .flat_map(|tx| tx.rct_signatures.iter())
            .collect::<Vec<_>>();

        if ringct::ringct::verify_multiple(&signatures).is_err() {
            return Err(failure::format_err!("Invalid RingCT signatures"));
        }
        Ok(())
    }
}
