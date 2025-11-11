use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use tracing::debug;

use crate::api::{error::Error, state::AppState};

#[derive(Clone)]
pub struct AuthenticatedUser(pub String);

/// Authentication middleware for protected routes
///
/// This middleware validates authentication creds.
#[tracing::instrument(level = tracing::Level::DEBUG, skip(state, request, next))]
pub async fn auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, Error> {
    debug!("Auth middleware invoked");

    let headers = request.headers();

    if let Some(username) = headers.get("x-auth-user").and_then(|v| v.to_str().ok())
        && let Some(key) = headers.get("x-auth-key").and_then(|v| v.to_str().ok())
    {
        if let Some(user) = state.sync.get_user(username)? {
            user.check(key)
                .map_err(|_| Error::Unauthorized("Invalid credentials".to_string()))?;
            let user = AuthenticatedUser(username.to_string());
            request.extensions_mut().insert(user);
            Ok(next.run(request).await)
        } else {
            Err(Error::UserNotFound(username.to_string()))
        }
    } else {
        Err(Error::Unauthorized("Missing credentials".to_string()))
    }
}
