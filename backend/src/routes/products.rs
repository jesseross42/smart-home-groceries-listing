//! A list item's products at Woolworths and Coles.
//!
//! The route only reads: it searches the stores and answers with what they
//! sell, cheapest per unit first. Choosing a product for the order is the
//! Order Optimisation feature's job.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use uuid::Uuid;

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::models::schemas::{ItemProductsQuery, ItemProductsResponse};
use crate::routes::extract::ValidatedQuery;
use crate::services::product_search::ProductSearchService;
use crate::state::AppState;

/// Routes under `/api/grocery-items/{id}/products`.
pub fn router() -> Router<AppState> {
    Router::new().route("/api/grocery-items/{item_id}/products", get(item_products))
}

/// `GET /api/grocery-items/{id}/products[?q=words]` — search every store
/// for the item, or for `q` (other words, chips not applied) priced at the
/// item's quantity. A blank `q` is the item's own search.
///
/// Always 200 once the item exists: a store that fails is reported inside
/// the body (`status`, `message`) while the other store's products show.
async fn item_products(
    State(state): State<AppState>,
    _user: AuthUser,
    Path(item_id): Path<Uuid>,
    ValidatedQuery(query): ValidatedQuery<ItemProductsQuery>,
) -> Result<Json<ItemProductsResponse>, ApiError> {
    let words = query.q.as_deref().map(str::trim).filter(|w| !w.is_empty());
    let search = ProductSearchService::new(&state.pool, &state.stores)
        .search_item_with(item_id, words)
        .await?;
    Ok(Json(search.into()))
}
