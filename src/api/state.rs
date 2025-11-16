use std::sync::Arc;

use crate::service::db::KorrosyncService;

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub sync: Arc<dyn KorrosyncService + Send + Sync>,
}
