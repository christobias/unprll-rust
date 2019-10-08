use failure::Error;

/// Type alias for the Result returned from functions in this crate
pub type Result<T> = std::result::Result<T, Error>;
