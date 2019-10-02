#![deny(missing_docs)]

//! Offers common functionality to binary crates (currently contains logging configuration)

mod config;
/// Functions for setting up the logging system
pub mod logger;

pub use config::Config;
