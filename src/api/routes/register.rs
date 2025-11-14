use crate::{
    api::{error::ApiError, state::AppState},
    model::User,
};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use serde_json::json;
use tracing::{Level, instrument};

/// Register Router - handles user registration
pub fn create_route() -> Router<AppState> {
    Router::new().route("/users/create", post(register))
}

#[instrument(level = Level::DEBUG, skip(payload, state))]
async fn register(
    State(state): State<AppState>,
    WithRejection(Json(payload), _): WithRejection<Json<RegisterUser>, ApiError>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    if (state.sync.get_user(&payload.username)?).is_some() {
        return Err(ApiError::ExistingUser(payload.username));
    }

    state
        .sync
        .add_user(&User::new(&payload.username, &payload.password).map_err(ApiError::HashError)?)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({"username": payload.username})),
    ))
}

#[derive(Deserialize, Debug)]
struct RegisterUser {
    username: String,
    password: String,
}

impl RegisterUser {
    fn validate(&self) -> Result<(), ApiError> {
        if self.username.is_empty() || self.password.is_empty() {
            return Err(ApiError::InvalidInput(
                "Username and password cannot be empty".into(),
            ));
        }

        Ok(())
    }
}
