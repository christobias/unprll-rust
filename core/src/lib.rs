use std::sync::{
    Arc,
    RwLock
};

use blockchain::Blockchain;
use common::Config;

/// # Cryptonote Core
///
/// This struct is a convenience wrapper around all the components
/// of Cryptonote (such as the blockchain and transaction mempool)
#[derive(Clone)]
pub struct CryptonoteCore {
    blockchain: Arc<RwLock<Blockchain>>,
}

impl CryptonoteCore {
    pub fn new(config: &Config) -> Self {
        let blockchain = Arc::from(RwLock::from(Blockchain::new(config).expect("Failed to initialize Blockchain")));
        let core = CryptonoteCore {
            blockchain,
        };
        core
    }
    pub fn blockchain(&self) -> Arc<RwLock<Blockchain>> {
        self.blockchain.clone()
    }
}
