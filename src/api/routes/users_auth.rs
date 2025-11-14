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
    #[serde(skip_serializing_if = "Option::is_none")]
    last_activity: Option<i64>,
}

/// Handler for GET /users/auth
///
/// Returns authentication status
#[tracing::instrument(
    skip_all,
    fields(
        correlation_id = %uuid::Uuid::new_v4(),
        username=username,
    )
)]
async fn get_auth_user(
    Extension(AuthenticatedUser(username, last_activity)): Extension<AuthenticatedUser>,
) -> Result<Json<AuthResponse>, StatusCode> {
    info!("User auth check requested");

    let response = AuthResponse {
        authorized: "OK".to_string(),
        username,
        last_activity,
    };

    Ok(Json(response))
}
