use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    // RPC
    #[structopt(long, default_value = "21150")]
    pub rpc_bind_port: u16,
}
