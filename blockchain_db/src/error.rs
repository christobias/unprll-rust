use std::error::Error as StdError;

#[derive(Debug)]
pub enum Error {
    GenericError(String),
    TransactionStartError(String),
    DBOpenError(String),
    DBCreateError(String),
    DBSyncError(String),
    DoesNotExist(String),
    Exists(String),
    Invalid(String)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::GenericError(message) => message,
            Error::TransactionStartError(message) => message,
            Error::DBOpenError(message) => message,
            Error::DBCreateError(message) => message,
            Error::DBSyncError(message) => message,
            Error::DoesNotExist(message) => message,
            Error::Exists(message) => message,
            Error::Invalid(message) => message
        }
    }
}
