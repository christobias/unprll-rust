use structopt::StructOpt;

use blockchain_db::Config as BlockchainDBConfig;

/// Config struct for the blockchain
#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    // Structopt does not allow doc comments for flattened structs
    #[allow(missing_docs)]
    #[structopt(flatten)]
    pub blockchain_db_config: BlockchainDBConfig,
}
