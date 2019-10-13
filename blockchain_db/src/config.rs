use std::path::PathBuf;

use structopt::StructOpt;

/// Configuration for BlockchainDB
#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case", name = "Unprll")]
pub struct Config {
    // Blockchain DB

    /// Type of database
    #[structopt(long, default_value = "memory")]
    pub db_type: String,

    /// Path where database files should be stored
    /// If unset, uses the default data directory
    #[structopt(long)]
    pub db_data_directory: Option<PathBuf>
}
