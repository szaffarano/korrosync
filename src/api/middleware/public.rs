use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use tracing::{Level, debug};

/// Public middleware for routes that don't require authentication
///
/// This middleware logs requests and passes them through without validation.
#[tracing::instrument(level = Level::DEBUG, skip(request, next))]
pub async fn public(request: Request, next: Next) -> Result<Response, StatusCode> {
    debug!(
        "Public route accessed: {} {}",
        request.method(),
        request.uri()
    );

    Ok(next.run(request).await)
}
