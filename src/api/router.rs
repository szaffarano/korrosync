use axum::{Router, http::StatusCode, middleware};
use tokio::{net::TcpListener, signal};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::{
    api::{
        middleware::{self as api_middleware, ratelimiter::rate_limiter_layer},
        routes,
        state::AppState,
    },
    error::Result,
};

pub fn app(state: AppState) -> Router {
    let public_routes = Router::new()
        .merge(routes::robots::create_route())
        .merge(routes::register::create_route())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn(api_middleware::public::public)),
        );

    let auth_routes = Router::new()
        .merge(routes::users_auth::create_route())
        .merge(routes::syncs_progress::create_route())
        .merge(routes::healthcheck::create_route())
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    api_middleware::auth::auth,
                )),
        );

    Router::new()
        .merge(public_routes)
        .merge(auth_routes)
        .fallback(|| async { StatusCode::NOT_FOUND })
        .layer(rate_limiter_layer())
        .with_state(state)
}

pub async fn serve(listener: TcpListener, state: AppState) -> Result<()> {
    info!("Server listening on {}", &listener.local_addr()?);

    axum::serve(
        listener,
        app(state).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}

/// Handle graceful shutdown signals
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let interrupt = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let interrupt = std::future::pending::<()>();

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = interrupt => {},
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
