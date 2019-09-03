use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    // Blockchain DB
    #[structopt(long, default_value = "memory")]
    pub db_type: String
}
