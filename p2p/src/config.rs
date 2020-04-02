use structopt::StructOpt;

/// Crypronote P2P configuration
#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    /// Port to bind the P2P listener
    #[structopt(long, default_value = "21149")]
    pub p2p_bind_port: u16,

    /// Address of node to connect to
    #[structopt(long)]
    pub connect_to: Option<String>,
}
