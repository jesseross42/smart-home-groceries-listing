//! The order planner: the ways to buy the committed list, with delivery.
//! Using one is `PUT /api/order-stores` (`routes/selections.rs`).

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{OrderOptionsQuery, OrderOptionsResponse};
use crate::routes::extract::ValidatedQuery;
use crate::services::order_plan::OrderPlanService;
use crate::state::AppState;

/// `GET /api/order-options`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/order-options", get(order_options))
}

/// `GET /api/order-options[?mode=&day=&within=]` — the options, ranked by `mode` or
/// the saved one, with delivery fees for `day` or the usual day. A store that cannot be asked does not fail the request; its
/// lines say so, as in the order review.
async fn order_options(
    State(state): State<AppState>,
    _user: AuthUser,
    ValidatedQuery(query): ValidatedQuery<OrderOptionsQuery>,
) -> Result<Json<OrderOptionsResponse>, ApiError> {
    let planned = OrderPlanService::new(&state.pool, &state.stores)
        .plan(query.mode, query.day_choice())
        .await?;
    Ok(Json(OrderOptionsResponse::new(planned)))
}
