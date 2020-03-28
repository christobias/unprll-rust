use structopt::StructOpt;

use blockchain::Config as BlockchainConfig;

/// CryptonoteCore configuration
#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    // We can't document flattened structs
    #[allow(missing_docs)]
    #[structopt(flatten)]
    pub blockchain_config: BlockchainConfig,
}
