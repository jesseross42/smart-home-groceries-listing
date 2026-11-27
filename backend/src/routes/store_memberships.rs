//! Settings › Member prices: whether the household is a member of each
//! store's rewards program, so member prices count.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{SaveStoreMembershipsBody, StoreMembershipsResponse};
use crate::routes::extract::ValidatedJson;
use crate::services::store_memberships::StoreMembershipsService;
use crate::state::AppState;

/// `GET` and `PUT /api/store-memberships`.
pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/store-memberships",
        get(read_memberships).put(save_memberships),
    )
}

/// `GET /api/store-memberships` — every store with a rewards program.
async fn read_memberships(
    State(state): State<AppState>,
    _user: AuthUser,
) -> Result<Json<StoreMembershipsResponse>, ApiError> {
    let memberships = StoreMembershipsService::new(&state.pool).read().await?;
    Ok(Json(memberships.into()))
}

/// `PUT /api/store-memberships` — saves the stores sent; 422 for a store
/// without a rewards program. Answers with every membership as it now
/// stands, which the web app keeps for its Undo.
async fn save_memberships(
    State(state): State<AppState>,
    user: AuthUser,
    ValidatedJson(body): ValidatedJson<SaveStoreMembershipsBody>,
) -> Result<Json<StoreMembershipsResponse>, ApiError> {
    let memberships = StoreMembershipsService::new(&state.pool)
        .save(&body.memberships(), &user.id)
        .await?;
    Ok(Json(memberships.into()))
}
