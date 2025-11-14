use std::net::SocketAddr;

use tokio::{net::TcpListener, signal};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{
    api::{middleware::ratelimiter::rate_limiter_layer, router::app, state::AppState},
    config::Config,
    error::Result,
    sync::service::KorrosyncService,
};

use crate::logging::init_logging;

pub mod api;
pub mod config;
pub mod error;
pub mod logging;
pub mod model;
pub mod sync;

pub async fn run_server(cfg: Config) -> Result<()> {
    init_logging();

    let listener = TcpListener::bind(cfg.server.address).await?;
    let state = AppState {
        sync: KorrosyncService::new(cfg.db.path)?,
    };

    let shutdown_token_cleanup = CancellationToken::new();
    let (rate_limiter, cleanup_task) = rate_limiter_layer(shutdown_token_cleanup.clone());

    let app = app(state)
        .layer(rate_limiter)
        .into_make_service_with_connect_info::<SocketAddr>();

    info!("Server listening on {}", &listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    // Cancel the rate limiter cleanup task and wait for it to finish
    shutdown_token_cleanup.cancel();
    cleanup_task.await.map_err(|e| {
        tracing::error!("Rate limiter cleanup task failed: {}", e);
        crate::error::Error::Custom("Rate limit cleanup task failed".to_string())
    })?;

    info!("Server shutdown complete");

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
