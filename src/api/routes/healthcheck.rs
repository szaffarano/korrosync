use axum::{Router, http::StatusCode, routing::get};
use tracing::{Level, instrument};

use crate::api::state::AppState;

/// Health Check Router - contains one single GET health endpoint, meant to be used for probes
pub fn create_route() -> Router<AppState> {
    Router::new().route("/healthcheck", get(get_health_check))
}

#[instrument(level = Level::DEBUG)]
async fn get_health_check() -> StatusCode {
    StatusCode::OK
}
