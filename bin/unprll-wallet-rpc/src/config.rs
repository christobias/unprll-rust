use structopt::StructOpt;

use bin_common::Config as BinCommonConfig;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    // RPC
    #[structopt(long)]
    pub rpc_bind_port: u16,

    #[structopt(long)]
    pub wallet_dir: std::path::PathBuf,

    #[structopt(long, default_value = "localhost:21150")]
    pub daemon_address: String,

    #[structopt(flatten)]
    pub bin_common_config: BinCommonConfig
}
