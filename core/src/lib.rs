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
    pub fn get_blockchain(&self) -> &Blockchain {
        &self.blockchain
    }
    pub fn get_blockchain_mut(&mut self) -> &mut Blockchain {
        &mut self.blockchain
    }
}
