//! Korrosync - KOReader synchronization server
//!
//! A Rust implementation of a synchronization server compatible with KOReader's sync
//! functionality. This server provides a self-hosted alternative for synchronizing reading
//! progress across multiple KOReader devices.
//!
//! # Architecture
//!
//! The crate is organized into three main layers:
//!
//! ## Model Layer ([`model`])
//!
//! Domain models representing the core business entities:
//! - [`model::User`] - User authentication and profile management
//! - [`model::Progress`] - Reading progress tracking for documents
//! - [`model::Error`] - Model-specific errors
//!
//! ## Service Layer ([`service`])
//!
//! Business logic and data persistence:
//! - [`service::db`] - Database abstraction with trait-based design
//! - [`service::db::KorrosyncServiceRedb`] - Default redb implementation
//! - [`service::error`] - Service-level error types
//!
//! ## API Layer ([`api`])
//!
//! HTTP API compatible with KOReader sync protocol:
//! - RESTful endpoints for user registration and authentication
//! - Progress synchronization endpoints
//! - Rate limiting and authentication middleware
//!
//! # Quick Start
//!
//! ```no_run
//! use korrosync::{config::Config, run_server};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load configuration from environment or defaults
//!     let config = Config::from_env();
//!
//!     // Start the server
//!     run_server(config).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Configuration
//!
//! The server can be configured via environment variables:
//! - `KORROSYNC_SERVER_ADDR` - Server bind address (default: 0.0.0.0:3000)
//! - `KORROSYNC_DB_PATH` - Database file path (default: data/db.redb)
//!
//! # KOReader Compatibility
//!
//! This server implements the KOReader synchronization API, allowing you to:
//! - Register user accounts
//! - Authenticate devices
//! - Synchronize reading progress across devices
//!
//! Configure your KOReader device to point to your server URL to start syncing.

use std::{net::SocketAddr, sync::Arc};

use color_eyre::eyre;
use tokio::{net::TcpListener, signal};
use tokio_util::sync::CancellationToken;
use tracing::info;

use crate::{
    api::{middleware::ratelimiter::rate_limiter_layer, router::app, state::AppState},
    config::Config,
    service::db::KorrosyncServiceRedb,
};

use crate::logging::init_logging;

pub mod api;
pub mod config;
pub mod logging;
pub mod model;
pub mod service;

pub async fn run_server(cfg: Config) -> eyre::Result<()> {
    init_logging();

    let listener = TcpListener::bind(cfg.server.address).await?;
    let state = AppState {
        sync: Arc::new(KorrosyncServiceRedb::new(cfg.db.path)?),
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
        e
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
