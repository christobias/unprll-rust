use structopt::StructOpt;

use bin_common::Config as BinCommonConfig;
use cryptonote_core::Config as CryptonoteCoreConfig;
use p2p::Config as P2PConfig;
use rpc::Config as RPCConfig;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    #[structopt(flatten)]
    pub bin_common_config: BinCommonConfig,

    #[structopt(flatten)]
    pub cryptonote_core_config: CryptonoteCoreConfig,

    #[structopt(flatten)]
    pub rpc_config: RPCConfig,

    #[structopt(flatten)]
    pub p2p_config: P2PConfig,
}
