use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    #[structopt(long, default_value = "21149")]
    pub p2p_bind_port: u16,

    #[structopt(long)]
    pub connect_to: Option<String>,
}
