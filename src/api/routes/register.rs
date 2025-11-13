use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};
use serde::Deserialize;
use serde_json::json;
use tracing::{Level, debug, instrument};

use crate::{
    api::{self, state::AppState},
    model::User,
};

/// Register Router - handles user registration
pub fn create_route() -> Router<AppState> {
    Router::new().route("/users/create", post(register))
}

#[instrument(level = Level::DEBUG, skip(payload, state))]
async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUser>,
) -> Result<impl IntoResponse, api::error::Error> {
    debug!("register requested for user: {}", payload.username);

    payload.validate()?;

    if (state.sync.get_user(&payload.username)?).is_some() {
        return Err(api::error::Error::ExistingUser(payload.username));
    }

    state
        .sync
        .add_user(&User::new(&payload.username, &payload.password)?)?;

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
    fn validate(&self) -> Result<(), api::error::Error> {
        if self.username.is_empty() || self.password.is_empty() {
            debug!("registration failed: username or key is empty");
            return Err(api::error::Error::InvalidInput(
                "Username and password cannot be empty".into(),
            ));
        }

        Ok(())
    }
}
