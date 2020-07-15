use blockchain_db::Error as BlockchainDBError;

/// Type alias for Blockchain operations that may result in an error
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for blockchain operations
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Returned when a transaction does not follow network semantics
    #[error("Transaction does not follow network semantics")]
    InvalidTransaction,

    /// Returned when a block is part of an alternative chain
    #[error("Block is from an alternative chain")]
    AltChainBlock,

    /// Returned when a block contains an unconfirmed transaction we haven't received
    #[error("Block contains an extraneous transaction")]
    ExtraneousTransaction,

    /// Returned when the blockchain DB returns an error
    #[error(transparent)]
    DBError(#[from] BlockchainDBError),
}
