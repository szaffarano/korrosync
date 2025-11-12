use axum::{Router, routing::get};
use tracing::{Level, debug};

use crate::api::state::AppState;

/// Create the robots.txt route
pub fn create_route() -> Router<AppState> {
    Router::new().route("/robots.txt", get(get_robots))
}

/// Handler for GET /robots.txt
///
/// Returns a robots.txt that disallows all crawling.
#[tracing::instrument(level = Level::DEBUG)]
async fn get_robots() -> &'static str {
    debug!("Robots.txt requested");

    "User-agent: *\nDisallow: /"
}
