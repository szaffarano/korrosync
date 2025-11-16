//! Error types for the service layer.
//!
//! This module defines error types that can occur during service-level operations,
//! such as database interactions, file I/O, or business logic failures.
//!
//! # Example
//!
//! ```no_run
//! use korrosync::service::error::ServiceError;
//! use korrosync::service::db::KorrosyncServiceRedb;
//!
//! // ServiceError is returned from service operations
//! let result = KorrosyncServiceRedb::new("invalid/path/db.redb");
//!
//! match result {
//!     Ok(service) => println!("Service created successfully"),
//!     Err(ServiceError::Io(e)) => eprintln!("I/O error: {}", e),
//!     Err(ServiceError::DB(e)) => eprintln!("Database error: {}", e),
//! }
//! ```

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ServiceError {
    // I/O errors that occur during file operations, such as:
    // - Creating database parent directories
    // - Reading or writing database files
    // - File permission errors
    #[error(transparent)]
    Io(#[from] std::io::Error),

    // Database-related errors from the storage layer, including:
    // - Failed database transactions
    // - Serialization/deserialization errors
    // - Database corruption or incompatibility
    // - Table creation or access failures
    #[error(transparent)]
    DB(Box<dyn std::error::Error + Send + Sync>),
}

impl ServiceError {
    pub fn db(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        ServiceError::DB(Box::new(e))
    }
}
