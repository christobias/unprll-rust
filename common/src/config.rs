use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    // Binaries
    #[structopt(long, default_value = "1")]
    pub log_level: u8,

    #[structopt(long)]
    pub data_directory: Option<PathBuf>,

    // Blockchain DB
    #[structopt(long, default_value = "memory")]
    pub db_type: String,

    // P2P
    #[structopt(long, default_value = "21149")]
    pub p2p_bind_port: u16,

    // RPC
    #[structopt(long, default_value = "21150")]
    pub rpc_bind_port: u16
}
