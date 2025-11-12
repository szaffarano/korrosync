use crate::sync::KorrosyncService;

/// Application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub sync: KorrosyncService,
}
