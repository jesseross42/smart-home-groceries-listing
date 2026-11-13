//! `/api/voice-list-changes` — the household's view of items removed or
//! reduced by voice, and Undo for one of them
//! (`docs/features/FEATURE_VOICE.md`).
//!
//! Needs a signed-in household member. The bridge's own routes are in
//! [`super::alexa_list_changes`]; the rules are shared, in
//! [`crate::services::voice_changes`].

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::VoiceListChangeEntry;
use crate::services::voice_changes::{self, RecentChange};
use crate::state::AppState;

/// Routes under `/api/voice-list-changes`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/voice-list-changes", get(list_recent))
        .route("/api/voice-list-changes/{id}/undo", post(undo_change))
}

/// `GET /api/voice-list-changes` — the newest changes, newest first.
async fn list_recent(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<Vec<VoiceListChangeEntry>>, ApiError> {
    let changes = voice_changes::recent(&state.pool).await?;
    Ok(Json(changes.into_iter().map(Into::into).collect()))
}

/// `POST /api/voice-list-changes/{id}/undo` — reverse that change.
///
/// 200 with the change, now undone. 404 unknown id; 409 when it is already
/// undone, older than 30 minutes, or its item was deleted or committed since.
async fn undo_change(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(id): Path<Uuid>,
) -> Result<Json<VoiceListChangeEntry>, ApiError> {
    let change = voice_changes::undo_one(&state.pool, id).await?;
    Ok(Json(
        RecentChange {
            change,
            can_undo: false,
        }
        .into(),
    ))
}
