use axum::http::StatusCode;
use tracing::{Level, instrument};

#[instrument(level = Level::DEBUG)]
pub async fn fallback() -> StatusCode {
    StatusCode::NOT_FOUND
}
