use axum::{Extension, Json, Router, http::StatusCode, routing::get};
use serde::Serialize;
use tracing::info;

use crate::api::{middleware::auth::AuthenticatedUser, state::AppState};

/// Create the user authentication route
pub fn create_route() -> Router<AppState> {
    Router::new().route("/users/auth", get(get_auth_user))
}

/// Response for authenticated user information
#[derive(Serialize)]
struct AuthResponse {
    authorized: String,
    username: String,
    last_activity: Option<i64>,
}

/// Handler for GET /users/auth
///
/// Returns authentication status
#[tracing::instrument(level = tracing::Level::DEBUG)]
async fn get_auth_user(
    Extension(AuthenticatedUser(username, last_activity)): Extension<AuthenticatedUser>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("User auth check requested: {username}");

    let response = AuthResponse {
        authorized: "OK".to_string(),
        username,
        last_activity,
    };

    Ok(Json(response))
}
