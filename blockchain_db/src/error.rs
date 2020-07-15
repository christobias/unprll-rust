/// Type alias for the Result returned from functions in this crate
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for Blockchain DB operations
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Returned when a block does not connect to the current main chain
    #[error("Block does not connect to our chain tail")]
    DoesNotConnect,

    /// Returned when a block has an invalid height
    #[error("Block has an invalid height")]
    InvalidHeight,

    /// Returned when a block/transaction/key image exists in the chain when it shouldn't
    #[error("Object exists in main chain")]
    Exists,

    /// Returned when a block/transaction/key image does not exist in the chain when it should
    #[error("Object does not exist in main chain")]
    DoesNotExist,

    /// Returned when the DB driver faces an internal issue
    #[error(transparent)]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}
