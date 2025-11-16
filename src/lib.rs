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
//! - `KORROSYNC_SERVER_ADDRESS` - Server bind address (default: 0.0.0.0:3000)
//! - `KORROSYNC_DB_PATH` - Database file path (default: data/db.redb)
//! - `KORROSYNC_USE_TLS` - Enable TLS/HTTPS (default: false, accepts: true/1/yes/on or false/0/no/off)
//! - `KORROSYNC_CERT_PATH` - Path to TLS certificate file in PEM format (default: tls/cert.pem)
//! - `KORROSYNC_KEY_PATH` - Path to TLS private key file in PEM format (default: tls/key.pem)
//!
//! # KOReader Compatibility
//!
//! This server implements the KOReader synchronization API, allowing you to:
//! - Register user accounts
//! - Authenticate devices
//! - Synchronize reading progress across devices
//!
//! Configure your KOReader device to point to your server URL to start syncing.

use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use axum_server::{Handle, tls_rustls::RustlsConfig};
use color_eyre::eyre::{self, Context};
use tokio::{signal, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

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

    let addr: SocketAddr = cfg
        .server
        .address
        .parse()
        .context("Error parsing binding address")?;

    let state = AppState {
        sync: Arc::new(KorrosyncServiceRedb::new(cfg.db.path).context("DB Init Error")?),
    };

    let shutdown_token_cleanup = CancellationToken::new();
    let (rate_limiter, cleanup_task) = rate_limiter_layer(shutdown_token_cleanup.clone());

    let app = app(state)
        .layer(rate_limiter)
        .into_make_service_with_connect_info::<SocketAddr>();

    let shutdown_handle = Handle::new();
    tokio::spawn(shutdown_signal(shutdown_handle.clone()));

    if cfg.server.use_tls {
        info!("TLS Server listening on {}", &addr);

        let tls_config = RustlsConfig::from_pem_file(
            PathBuf::from(cfg.server.cert_path),
            PathBuf::from(cfg.server.key_path),
        )
        .await
        .context("Error loading TLS keys")?;

        axum_server::bind_rustls(addr, tls_config)
            .handle(shutdown_handle)
            .serve(app)
            .await
            .context("Failed to start TLS server")?;
    } else {
        info!("Server listening on {}", &addr);

        axum_server::bind(addr)
            .handle(shutdown_handle)
            .serve(app)
            .await
            .context("Failed to start server")?;
    }

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
///
/// A background task is spawned to listen for shutdown signals (Ctrl-C, SIGINT, SIGTERM).
/// Then call the handle's `graceful_shutdown` method to initiate a graceful shutdown of the
/// server.
#[instrument(fields(graceful_shutdown), skip(handle))]
async fn shutdown_signal(handle: Handle) {
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
        _ = interrupt => info!("Got SIGINT"),
        _ = ctrl_c => info!("Got Ctrl-C"),
        _ = terminate => info!("Got SIGTERM"),
    }

    info!("Server is shutting down...");

    handle.graceful_shutdown(Some(Duration::from_secs(30)));

    // 1 min
    let mut retries = 60;
    loop {
        retries -= 1;
        if retries == 0 {
            tracing::warn!("Forcing shutdown with live connections");
            break;
        }

        sleep(Duration::from_secs(1)).await;
        let connections = handle.connection_count();
        tracing::info!("{connections} live connections left");
        if connections == 0 {
            break;
        }
    }
}
