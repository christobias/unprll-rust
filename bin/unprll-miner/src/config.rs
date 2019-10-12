use structopt::StructOpt;

use bin_common::Config as BinCommonConfig;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    #[structopt(flatten)]
    pub bin_common_config: BinCommonConfig,

    #[structopt(long, default_value = "10")]
    pub check_interval: u64,

    #[structopt(long, default_value = "localhost:21150")]
    pub daemon_address: String,

    #[structopt(long)]
    pub miner_address: String
}
