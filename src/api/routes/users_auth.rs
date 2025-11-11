use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use serde::Serialize;
use tracing::info;

use crate::api::state::AppState;

/// Create the user authentication route
pub fn create_route() -> Router<AppState> {
    Router::new().route("/users/auth", get(get_auth_user))
}

/// Response for authenticated user information
#[derive(Serialize)]
struct AuthResponse {
    authorized: String,
}

/// Handler for GET /users/auth
///
/// Returns authentication status
#[tracing::instrument(level = tracing::Level::DEBUG, skip(_state))]
async fn get_auth_user(State(_state): State<AppState>) -> Result<Json<AuthResponse>, StatusCode> {
    info!("User auth check requested");

    let response = AuthResponse {
        authorized: "OK".to_string(),
    };

    Ok(Json(response))
}
