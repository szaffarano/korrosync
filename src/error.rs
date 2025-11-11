use thiserror::Error;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error("{0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database open error: {0}")]
    DbOpen(#[from] redb::DatabaseError),

    #[error("Database table error: {0}")]
    DbTable(#[from] redb::TableError),

    #[error("Database transaction error: {0}")]
    DbTx(#[from] redb::TransactionError),

    #[error("Database storage error: {0}")]
    DbStorage(#[from] redb::StorageError),

    #[error("Database commit error: {0}")]
    DbCommit(#[from] redb::CommitError),

    #[error("Password hashing error: {0}")]
    Hash(#[from] argon2::password_hash::Error),
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Custom(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Custom(s.to_string())
    }
}
