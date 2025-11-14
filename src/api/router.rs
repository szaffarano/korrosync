use axum::{Router, middleware};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::api::{
    middleware::{self as api_middleware},
    routes,
    state::AppState,
};

pub fn app(state: AppState) -> Router {
    let public_routes = Router::new()
        .merge(routes::robots::create_route())
        .merge(routes::register::create_route())
        .layer(ServiceBuilder::new().layer(middleware::from_fn(api_middleware::public::public)));

    let auth_routes = Router::new()
        .merge(routes::users_auth::create_route())
        .merge(routes::syncs_progress::create_route())
        .merge(routes::healthcheck::create_route())
        .layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(
            state.clone(),
            api_middleware::auth::auth,
        )));

    Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .fallback(routes::fallback::fallback)
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
