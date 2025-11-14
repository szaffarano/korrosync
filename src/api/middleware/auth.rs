use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tracing::debug;

use crate::api::{error::ApiError, state::AppState};

#[derive(Clone, Debug)]
pub struct AuthenticatedUser(pub String, pub Option<i64>);

/// Authentication middleware for protected routes
///
/// This middleware validates authentication creds.
#[tracing::instrument(level = tracing::Level::DEBUG, skip(state, request, next))]
pub async fn auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    debug!("Auth middleware invoked");

    let headers = request.headers();

    if let Some(username) = headers.get("x-auth-user").and_then(|v| v.to_str().ok())
        && let Some(key) = headers.get("x-auth-key").and_then(|v| v.to_str().ok())
    {
        if let Some(mut user) = state.sync.get_user(username)? {
            // Check password first - if this fails, it's an authentication error
            user.check(key)
                .map_err(|_| ApiError::Unauthorized("Invalid credentials".to_string()))?;

            // Update last activity - if this fails, it's a database error
            user.touch();
            state.sync.add_user(&user)?;

            let user = AuthenticatedUser(username.to_string(), user.last_activity());
            request.extensions_mut().insert(user);
            Ok(next.run(request).await)
        } else {
            Err(ApiError::Unauthorized("Invalid credentials".to_string()))
        }
    } else {
        Err(ApiError::Unauthorized("Missing credentials".to_string()))
    }
}
