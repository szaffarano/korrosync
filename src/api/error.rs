use axum::{Json, http::StatusCode, response::IntoResponse};
use serde_json::json;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("User '{0}' already exists")]
    ExistingUser(String),

    #[error("User '{0}' not found")]
    UserNotFound(String),

    #[error("Internal server error")]
    Internal(#[from] crate::error::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!(">>>>>>. into respose: {self:?}");
        // Log internal errors with full context for debugging
        let (status, message) = match &self {
            Error::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            Error::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.to_string()),
            Error::ExistingUser(msg) => (StatusCode::PAYMENT_REQUIRED, msg.to_string()),
            Error::UserNotFound(msg) => (StatusCode::NOT_FOUND, msg.to_string()),
            Error::Internal(err) => {
                error!("Internal error: {:?}", err);

                if matches!(err, crate::error::Error::NotFound(_)) {
                    // not a real error, return an empty json (it seems it's expected by the
                    // koreader client)
                    return (StatusCode::OK, Json(json!({}))).into_response();
                }
                // Log the actual error with full context
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}
