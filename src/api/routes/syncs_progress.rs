use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::{get, put},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info};

use crate::api::{middleware::auth::AuthenticatedUser, state::AppState};
use crate::sync::service::Progress;

/// Create the syncs progress routes
pub fn create_route() -> Router<AppState> {
    Router::new()
        .route("/syncs/progress", put(update_progress))
        .route("/syncs/progress/{doc}", get(get_progress))
}

/// Request body for updating sync progress
#[derive(Debug, Deserialize)]
struct UpdateProgressRequest {
    pub device_id: String,
    pub device: String,
    pub document: String,
    pub percentage: f32,
    pub progress: String,
}

/// Response for sync progress
#[derive(Serialize)]
struct ProgressResponse {
    pub device_id: String,
    pub device: String,
    pub document: String,
    pub percentage: f32,
    pub progress: String,
    pub timestamp: u64,
}

/// Handler for PUT /syncs/progress
///
/// Updates the synchronization progress for a document
#[tracing::instrument(level = tracing::Level::DEBUG, skip(state))]
async fn update_progress(
    State(state): State<AppState>,
    Extension(AuthenticatedUser(user, _)): Extension<AuthenticatedUser>,
    Json(payload): Json<UpdateProgressRequest>,
) -> Result<impl IntoResponse, crate::api::error::Error> {
    debug!("Updating sync progress");

    let (doc, ts) = state
        .sync
        .update_progress(user, payload.document.clone(), payload.into())?;

    Ok(Json(json!({
        "document": doc,
        "timestamp": ts,
    }))
    .into_response())
}

/// Handler for GET /syncs/progress/{doc}
///
/// Returns the synchronization progress for a specific document
#[tracing::instrument(level = tracing::Level::DEBUG, skip(state))]
async fn get_progress(
    State(state): State<AppState>,
    Extension(AuthenticatedUser(user, _)): Extension<AuthenticatedUser>,
    Path(doc): Path<String>,
) -> Result<Json<ProgressResponse>, crate::api::error::Error> {
    info!("Getting sync progress for doc: {}", doc);

    let progress = state.sync.get_progress(user, doc.clone())?;

    Ok(Json(ProgressResponse {
        document: doc,
        ..progress.into()
    }))
}

impl From<UpdateProgressRequest> for Progress {
    fn from(value: UpdateProgressRequest) -> Self {
        Self {
            device_id: value.device_id,
            device: value.device,
            percentage: value.percentage,
            progress: value.progress,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

impl From<Progress> for ProgressResponse {
    fn from(value: Progress) -> Self {
        Self {
            device_id: value.device_id,
            device: value.device,
            document: "".to_string(),
            percentage: value.percentage,
            progress: value.progress,
            timestamp: value.timestamp,
        }
    }
}
