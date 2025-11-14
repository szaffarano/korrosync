use axum::{
    Json,
    extract::rejection::{JsonRejection, PathRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;

use crate::sync::error::ServiceError;

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

    #[error("User '{0}' already exists")]
    HashError(argon2::password_hash::Error),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),
}

impl From<ServiceError> for ApiError {
    fn from(value: ServiceError) -> Self {
        match value {
            all @ ServiceError::Io(_) => ApiError::Service(all),

            all @ ServiceError::NotFound(_) => {
                ApiError::NotFound(ServiceError::NotFound(format!("{all}")))
            }

            all @ ServiceError::DB(_) => ApiError::Service(all),
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
                StatusCode::BAD_REQUEST,
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
            ApiError::HashError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiErrorPayload {
                    code: "hash_error",
                    message: format!("{err}"),
                },
            ),
            ApiError::Unauthorized(err) => (
                StatusCode::UNAUTHORIZED,
                ApiErrorPayload {
                    code: "unauthorized",
                    message: err.to_string(),
                },
            ),
        };

        (status, Json(payload)).into_response()
    }
}
