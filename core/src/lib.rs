use blockchain::Blockchain;
use txpool::TXPool;

mod config;
pub use config::Config;

/// # Cryptonote Core
///
/// This struct is a convenience wrapper around all the components
/// of Cryptonote (such as the blockchain and transaction mempool)
pub struct CryptonoteCore {
    blockchain: Blockchain,
    txpool: TXPool
}

impl CryptonoteCore {
    pub fn new(config: &Config) -> Self {
        let blockchain = Blockchain::new(&config.blockchain_config).expect("Failed to initialize Blockchain");
        let txpool     = TXPool::new();
        CryptonoteCore {
            blockchain,
            txpool
        }
    }
    pub fn blockchain(&self) -> &Blockchain {
        &self.blockchain
    }
    pub fn blockchain_mut(&mut self) -> &mut Blockchain {
        &mut self.blockchain
    }
    pub fn txpool(&self) -> &TXPool {
        &self.txpool
    }
}
