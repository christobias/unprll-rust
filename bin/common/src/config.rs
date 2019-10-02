use std::path::PathBuf;
use structopt::StructOpt;

/// Configuration for common systems
#[derive(StructOpt, Debug)]
#[structopt(rename_all = "kebab-case")]
pub struct Config {
    /// Sets the log level for the logger
    /// The levels correspond to the following:
    ///
    ///   0 - Warn
    ///   1 - Info
    ///   2 - Debug
    ///   3 - Trace
    #[structopt(long, default_value = "1")]
    pub log_level: u8,

    /// Sets the data directory to be used
    /// If unset, the default data directory is used
    #[structopt(long)]
    pub data_directory: Option<PathBuf>
}
