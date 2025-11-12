use crate::{
    api::{router, state::AppState},
    config::Config,
    error::Result,
    sync::KorrosyncService,
};
use tokio::net::TcpListener;

use crate::logging::init_logging;

pub mod api;
pub mod config;
pub mod error;
pub mod logging;
pub mod sync;

pub async fn run_server(cfg: Config) -> Result<()> {
    init_logging();

    let listener = TcpListener::bind(cfg.server.address).await?;

    let state = AppState {
        sync: KorrosyncService::new(cfg.db.path)?,
    };

    router::serve(listener, state).await
}
