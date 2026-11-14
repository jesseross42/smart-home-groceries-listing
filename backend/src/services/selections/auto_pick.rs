//! Choosing an item's products automatically when it reaches the list.
//!
//! The household asked for this (2026-11-15): every new item gets, at each
//! store, the cheapest product per unit that is available and that no
//! member dislikes (unless overridden for the item). The search answer is
//! already ordered that way ([`crate::services::product_search`]). The
//! household changes any pick on the Select item page. An item that already
//! has a choice is left alone. A store that fails, or offers nothing
//! suitable, is skipped; picking never fails the add.

use sqlx::PgPool;
use uuid::Uuid;

use super::{repository, SelectionService};
use crate::error::ApiError;
use crate::services::dislikes::skip::DislikeScope;
use crate::services::dislikes::DislikeService;
use crate::services::product_search::ProductSearchService;
use crate::services::stores::{Store, StoreClients};

/// Picks the item's products at every store, when `enabled`
/// (`AUTO_PICK_PRODUCTS`). Logs, never fails.
pub async fn pick_for_new_item(
    enabled: bool,
    pool: &PgPool,
    stores: &StoreClients,
    item_id: Uuid,
    user_id: &str,
) {
    if !enabled {
        return;
    }
    if let Err(err) = try_pick(pool, stores, item_id, user_id).await {
        tracing::warn!(%item_id, %err, "automatic product pick failed");
    }
}

async fn try_pick(
    pool: &PgPool,
    stores: &StoreClients,
    item_id: Uuid,
    user_id: &str,
) -> Result<(), ApiError> {
    let already = repository::list_every(pool)
        .await?
        .iter()
        .any(|s| s.grocery_item_id == item_id);
    if already {
        return Ok(());
    }
    let search = ProductSearchService::new(pool, stores)
        .search_item(item_id)
        .await?;
    let book = DislikeService::new(pool).book_for_item(item_id).await?;
    let picks: Vec<(Store, String)> = search
        .stores
        .iter()
        .filter(|outcome| outcome.error.is_none())
        .filter_map(|outcome| {
            outcome
                .products
                .iter()
                .find(|p| {
                    p.product.available
                        && p.total_price.is_some()
                        && !book.skips(
                            outcome.store,
                            &p.product.product_id,
                            DislikeScope::Household,
                        )
                })
                .map(|p| (outcome.store, p.product.product_id.clone()))
        })
        .collect();
    let service = SelectionService::new(pool, stores);
    for (store, product_id) in picks {
        if let Err(err) = service
            .choose(item_id, store, &product_id, false, user_id)
            .await
        {
            tracing::warn!(%item_id, %store, product_id, %err, "automatic pick refused");
        } else {
            tracing::info!(%item_id, %store, product_id, "product picked automatically");
        }
    }
    Ok(())
}
