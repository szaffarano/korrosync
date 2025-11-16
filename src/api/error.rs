//! Error handling for the API layer.
//!
//! This module defines error types for HTTP API operations and implements conversion
//! from lower-level errors (model, service) to HTTP responses with appropriate status
//! codes and error payloads.
//!
//! # Error Types
//!
//! The [`ApiError`] enum represents all possible error conditions in the API layer:
//!
//! - **Path/JSON Rejection**: Invalid request parameters or malformed JSON
//! - **Service Errors**: Database or I/O failures from the service layer
//! - **Not Found**: Resource not found (404)
//! - **Invalid Input**: Validation failures (e.g., empty username/password)
//! - **Existing User**: Attempting to create a duplicate user (409 Conflict)
//! - **Unauthorized**: Authentication failures (401)
//! - **Runtime**: Unexpected errors
//!
//! # HTTP Status Code Mapping
//!
//! Errors are automatically converted to HTTP responses with appropriate status codes:
//!
//! | Error Type | HTTP Status |
//! |-----------|-------------|
//! | PathRejection | 400 Bad Request |
//! | JsonRejection | 422 Unprocessable Entity |
//! | Service | 500 Internal Server Error |
//! | NotFound | 404 Not Found |
//! | InvalidInput | 400 Bad Request |
//! | ExistingUser | 402 Payment Required (keeps KOReader return code (?)) |
//! | Unauthorized | 401 Unauthorized |
//! | Runtime | 500 Internal Server Error |
//!
//! # Error Response Format
//!
//! All errors return a consistent JSON payload:
//!
//! ```json
//! {
//!   "code": "ERROR_CODE",
//!   "message": "Human-readable error message"
//! }
//! ```
//!
//! # Example
//!
//! ```no_run
//! use korrosync::api::error::ApiError;
//!
//! // Errors are automatically converted to HTTP responses
//! fn example_handler() -> Result<(), ApiError> {
//!     // This will become a 400 Bad Request response
//!     return Err(ApiError::InvalidInput("Username cannot be empty".to_string()));
//! }
//! ```

use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

use crate::{model, service::error::ServiceError};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorPayload {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("{0}")]
    PathRejection(#[from] PathRejection),

    #[error("{0}")]
    JsonRejection(#[from] JsonRejection),

    #[error("{0}")]
    Service(ServiceError),

    #[error("{0}")]
    NotFound(ServiceError),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("User '{0}' already exists")]
    ExistingUser(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error(transparent)]
    Runtime(Box<dyn std::error::Error + Send + Sync>),
}

impl From<ServiceError> for ApiError {
    fn from(value: ServiceError) -> Self {
        match value {
            all @ ServiceError::Io(_) => ApiError::Service(all),
            all @ ServiceError::DB(_) => ApiError::Service(all),
        }
    }
}

impl From<model::Error> for ApiError {
    fn from(value: model::Error) -> Self {
        match value {
            model::Error::Runtime(e) => ApiError::Runtime(e),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, payload) = match self {
            ApiError::PathRejection(_) => (
                StatusCode::BAD_REQUEST,
                ApiErrorPayload {
                    code: "bad_request",
                    message: "Invalid path parameter".to_string(),
                },
            ),
            ApiError::JsonRejection(ref e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ApiErrorPayload {
                    code: "bad_request",
                    message: e.to_string(),
                },
            ),
            ApiError::Service(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorPayload {
                    code: "io_failure",
                    message: format!("{err}"),
                },
            ),
            ApiError::NotFound(err) => (
                StatusCode::NOT_FOUND,
                ApiErrorPayload {
                    code: "not_found",
                    message: format!("{err}"),
                },
            ),
            all @ ApiError::InvalidInput(_) => (
                StatusCode::BAD_REQUEST,
                ApiErrorPayload {
                    code: "invalid_input",
                    message: all.to_string(),
                },
            ),
            all @ ApiError::ExistingUser(_) => (
                StatusCode::PAYMENT_REQUIRED,
                ApiErrorPayload {
                    code: "existing_user",
                    message: all.to_string(),
                },
            ),
            ApiError::Unauthorized(err) => (
                StatusCode::UNAUTHORIZED,
                ApiErrorPayload {
                    code: "unauthorized",
                    message: err.to_string(),
                },
            ),
            ApiError::Runtime(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorPayload {
                    code: "runtime_error",
                    message: err.to_string(),
                },
            ),
        };

        (status, Json(payload)).into_response()
    }
}

impl ApiError {
    pub fn runtime(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        ApiError::Runtime(Box::new(e))
    }
}
