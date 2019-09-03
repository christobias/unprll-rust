use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    #[structopt(long, default_value = "1")]
    pub log_level: u8,

    #[structopt(long)]
    pub data_directory: Option<PathBuf>
}
