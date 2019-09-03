use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    // P2P
    #[structopt(long, default_value = "21149")]
    pub p2p_bind_port: u16
}
