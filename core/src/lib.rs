#![deny(missing_docs)]
//! Core module to bind all components of a Cryptonote coin

use blockchain::Blockchain;
pub use blockchain::EmissionCurve;
use txpool::TXPool;

mod config;
pub use config::Config;

/// # Cryptonote Core
///
/// This struct is a convenience wrapper around all the components
/// of Cryptonote (such as the blockchain and transaction mempool)
pub struct CryptonoteCore<TCoin>
where
    TCoin: EmissionCurve,
{
    blockchain: Blockchain<TCoin>,
    txpool: TXPool,
}

impl<TCoin> CryptonoteCore<TCoin>
where
    TCoin: EmissionCurve,
{
    /// Creates a new CryptonoteCore with the given configuration
    pub fn new(coin_definition: TCoin, config: &Config) -> Self {
        let blockchain = Blockchain::new(coin_definition, &config.blockchain_config)
            .expect("Failed to initialize Blockchain");
        let txpool = TXPool::new();
        CryptonoteCore { blockchain, txpool }
    }
    /// Get a reference to the underlying blockchain
    pub fn blockchain(&self) -> &Blockchain<TCoin> {
        &self.blockchain
    }
    /// Get a mutable reference to the underlying blockchain
    pub fn blockchain_mut(&mut self) -> &mut Blockchain<TCoin> {
        &mut self.blockchain
    }
    /// Get a reference to the transaction mempool
    pub fn txpool(&self) -> &TXPool {
        &self.txpool
    }
}
