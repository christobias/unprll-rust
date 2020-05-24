use blockchain_db::Error as BlockchainDBError;

/// Type alias for Blockchain operations that may result in an error
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for blockchain operations
#[derive(Fail, Debug)]
pub enum Error {
    /// Returned when a block's genesis transaction does not follow network semantics
    #[fail(display = "Block's genesis transaction does not follow network semantics")]
    InvalidGenesisTransaction,

    /// Returned when a block is part of an alternative chain
    #[fail(display = "Block is from an alternative chain")]
    AltChainBlock,

    /// Returned when the blockchain DB returns an error
    #[fail(display = "{}", _0)]
    DBError(BlockchainDBError),
}

impl From<BlockchainDBError> for Error {
    fn from(error: BlockchainDBError) -> Self {
        Self::DBError(error)
    }
}
