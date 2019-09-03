use structopt::StructOpt;

use blockchain_db::Config as BlockchainDBConfig;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    #[structopt(flatten)]
    pub blockchain_db_config: BlockchainDBConfig
}
