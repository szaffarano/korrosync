use axum::{Router, http::StatusCode, routing::get};
use tracing::{Level, debug, instrument};

use crate::api::state::AppState;

/// Health Check Router - contains one single GET health endpoint, meant to be used for probes
pub fn create_route() -> Router<AppState> {
    Router::new().route("/healthcheck", get(get_health_check))
}

#[instrument(level = Level::DEBUG)]
async fn get_health_check() -> StatusCode {
    debug!("health-check requested");
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::api::routes::healthcheck::get_health_check;

    #[tokio::test]
    async fn test_health_check() {
        let status = get_health_check().await;

        assert_eq!(status, StatusCode::OK);
    }
}
