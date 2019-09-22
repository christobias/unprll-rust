use structopt::StructOpt;

use bin_common::Config as BinCommonConfig;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    // RPC
    #[structopt(long)]
    pub rpc_bind_port: u16,

    #[structopt(flatten)]
    pub bin_common_config: BinCommonConfig
}
