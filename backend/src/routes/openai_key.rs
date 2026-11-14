//! The OpenAI triage key on Settings › Help: save, remove, undo.
//!
//! No route returns the key. Each answers with whether one is set.

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::config::TriageProvider;
use crate::error::ApiError;
use crate::models::schemas::{OpenAiKeyStatusResponse, SaveOpenAiKeyBody};
use crate::routes::extract::ValidatedJson;
use crate::services::openai_key::{KeyStatus, OpenAiKeyService};
use crate::state::AppState;

/// Routes under `/api/triage/openai-key`.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/triage/openai-key", get(show).put(save).delete(remove))
        .route("/api/triage/openai-key/undo", post(undo))
}

fn service(state: &AppState) -> OpenAiKeyService<'_> {
    OpenAiKeyService::new(&state.pool, state.encryptor.as_ref())
}

fn answer(state: &AppState, status: KeyStatus) -> Json<OpenAiKeyStatusResponse> {
    let triage = &state.settings.triage;
    let from_environment = triage.provider == TriageProvider::OpenAi && !triage.api_key.is_empty();
    Json(OpenAiKeyStatusResponse::new(status, from_environment))
}

/// `GET /api/triage/openai-key` — whether a key is set.
async fn show(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<OpenAiKeyStatusResponse>, ApiError> {
    let status = service(&state).status().await?;
    Ok(answer(&state, status))
}

/// `PUT /api/triage/openai-key` — saves the key encrypted; 422 when empty.
async fn save(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<SaveOpenAiKeyBody>,
) -> Result<Json<OpenAiKeyStatusResponse>, ApiError> {
    let status = service(&state).save(&body.api_key, &user.id).await?;
    Ok(answer(&state, status))
}

/// `DELETE /api/triage/openai-key` — removes the saved key.
async fn remove(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<OpenAiKeyStatusResponse>, ApiError> {
    let status = service(&state).remove(&user.id).await?;
    Ok(answer(&state, status))
}

/// `POST /api/triage/openai-key/undo` — undoes the last save or removal.
async fn undo(
    State(state): State<AppState>,
    user: AuthUser,
) -> Result<Json<OpenAiKeyStatusResponse>, ApiError> {
    let status = service(&state).undo(&user.id).await?;
    Ok(answer(&state, status))
}
