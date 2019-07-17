use std::sync::RwLock;

use blockchain::Blockchain;
use common::Config;

pub struct CryptonoteCore {
    blockchain: Blockchain
}

impl CryptonoteCore {
    pub fn new(config: &Config) -> Self {
        let core = CryptonoteCore {
            blockchain: Blockchain::new(config).expect("Failed to initialize Blockchain")
        };
        core
    }
}
