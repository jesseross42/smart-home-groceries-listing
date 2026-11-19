//! Bought products: the one record per store product that the spending
//! analysis takes a product's category from (`bought_products`).
//!
//! The purchase history owns these rows, and this module holds every rule
//! for when one changes:
//!
//! * [`keep`] — saving a purchase writes its product's details, unless a
//!   newer purchase of that product is saved already;
//! * [`refresh`] — after Undo, each product takes the details of its newest
//!   purchase left, or goes when none is left;
//! * [`recategorise`] — "Sort categories again" works out every product's
//!   category with today's rules.
//!
//! A purchase row keeps its own copy of the details as the snapshot of what
//! was bought that day; totals are only ever read from purchases.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};

use super::category;
use super::product_repository::{self as repository, ProductDetails};
use super::repository::NewPurchase;
use crate::error::ApiError;
use crate::services::stores::Store;

/// Writes the product of `purchase`, bought at `bought_at`. Runs before the
/// purchase itself is written, which needs its product's row.
pub async fn keep(
    tx: &mut Transaction<'_, Postgres>,
    purchase: &NewPurchase<'_>,
    bought_at: DateTime<Utc>,
) -> Result<(), ApiError> {
    let details = ProductDetails {
        store: purchase.store,
        product_id: purchase.product_id,
        product_name: purchase.product_name,
        brand: purchase.brand,
        package_size: purchase.package_size,
        store_category: purchase.store_category,
        category: purchase.category,
        category_source: purchase.category_source,
        refreshed_at: bought_at,
    };
    repository::upsert_if_newer(&mut **tx, &details).await
}

/// Brings each of `products` back in step with its purchases, once some
/// were deleted: the newest purchase left gives the details, and a product
/// with no purchase left is deleted.
pub async fn refresh(
    tx: &mut Transaction<'_, Postgres>,
    products: &[(Store, String)],
) -> Result<(), ApiError> {
    for (store, product_id) in products {
        let Some(newest) = repository::newest_purchase(&mut **tx, *store, product_id).await? else {
            repository::delete(&mut **tx, *store, product_id).await?;
            continue;
        };
        let found = category::categorise(newest.store_category.as_deref(), &newest.product_name);
        let details = ProductDetails {
            store: *store,
            product_id,
            product_name: &newest.product_name,
            brand: newest.brand.as_deref(),
            package_size: newest.package_size.as_deref(),
            store_category: newest.store_category.as_deref(),
            category: found.category,
            category_source: found.source,
            refreshed_at: newest.bought_at,
        };
        repository::overwrite(&mut **tx, &details).await?;
    }
    Ok(())
}

/// How many products "Sort categories again" looked at, and moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Recategorised {
    pub products: usize,
    pub changed: usize,
}

/// Works out every product's category again with today's rules.
pub async fn recategorise(pool: &PgPool) -> Result<Recategorised, ApiError> {
    let mut tx = pool.begin().await?;
    let inputs = repository::category_inputs(&mut *tx).await?;
    let mut changed = 0;
    for input in &inputs {
        let found = category::categorise(input.store_category.as_deref(), &input.product_name);
        let store = input.store;
        let id = &input.product_id;
        if repository::set_category(&mut *tx, store, id, found.category, found.source).await? {
            changed += 1;
        }
    }
    tx.commit().await?;
    Ok(Recategorised {
        products: inputs.len(),
        changed,
    })
}
