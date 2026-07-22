//! Error types for the model layer.
//!
//! This module defines error types that can occur during model operations,
//! such as user creation, password hashing, or validation failures.
//!

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Runtime(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn runtime(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        Error::Runtime(Box::new(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_boxes_underlying_error() {
        let err = Error::runtime(std::io::Error::other("boom"));
        assert!(matches!(err, Error::Runtime(_)));
        assert_eq!(err.to_string(), "boom");
    }
}
