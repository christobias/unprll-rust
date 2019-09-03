use structopt::StructOpt;

use blockchain::Config as BlockchainConfig;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    #[structopt(flatten)]
    pub blockchain_config: BlockchainConfig
}
