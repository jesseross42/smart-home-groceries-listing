//! The reads of `purchases` that the product picker and the spending
//! analysis need. Writes are in [`super::repository`].
//!
//! Each purchase is read with its product's category from `bought_products`
//! (see [`super::products`]), which owns it.
//!
//! Optional filters are bound as `NULL` and tested in the statement
//! (`$1::timestamptz IS NULL OR ...`), so every statement stays a literal.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

use super::repository::{purchase_columns, purchases_and_products};
use crate::error::ApiError;
use crate::models::db::{ProductKey, Purchase};
use crate::services::stores::Store;

/// What the spending analysis narrows purchases to. `None` = no limit.
#[derive(Debug, Default)]
pub struct PurchaseFilter<'a> {
    /// Bought at or after this instant.
    pub from: Option<DateTime<Utc>>,
    /// Bought before this instant.
    pub until: Option<DateTime<Utc>>,
    pub store: Option<Store>,
    /// Part of the item's key (lowercase, spaces collapsed).
    pub item_key_part: Option<&'a str>,
    pub category: Option<&'a str>,
}

/// Every purchase of the given products, newest first.
pub async fn of_products<'e, E>(
    executor: E,
    products: &[ProductKey],
) -> Result<Vec<Purchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    let stores: Vec<&str> = products.iter().map(|key| key.store.as_str()).collect();
    let ids: Vec<&str> = products.iter().map(|key| key.product_id.as_str()).collect();
    Ok(sqlx::query_as::<_, Purchase>(concat!(
        "SELECT ",
        purchase_columns!(),
        purchases_and_products!(),
        " WHERE (p.store, p.product_id) IN (SELECT * FROM unnest($1::text[], $2::text[]))
         ORDER BY p.bought_at DESC, p.id"
    ))
    .bind(&stores)
    .bind(&ids)
    .fetch_all(executor)
    .await?)
}

/// Every purchase the filter lets through, oldest first.
pub async fn matching<'e, E>(
    executor: E,
    filter: &PurchaseFilter<'_>,
) -> Result<Vec<Purchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, Purchase>(concat!(
        "SELECT ",
        purchase_columns!(),
        purchases_and_products!(),
        " WHERE ($1::timestamptz IS NULL OR p.bought_at >= $1)
           AND ($2::timestamptz IS NULL OR p.bought_at < $2)
           AND ($3::text IS NULL OR p.store = $3)
           AND ($4::text IS NULL OR strpos(p.item_key, $4) > 0)
           AND ($5::text IS NULL OR bp.category = $5)
         ORDER BY p.bought_at, p.id"
    ))
    .bind(filter.from)
    .bind(filter.until)
    .bind(filter.store.map(Store::as_str))
    .bind(filter.item_key_part)
    .bind(filter.category)
    .fetch_all(executor)
    .await?)
}

/// Every purchase for one item, oldest first.
pub async fn of_item<'e, E>(executor: E, item_key: &str) -> Result<Vec<Purchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, Purchase>(concat!(
        "SELECT ",
        purchase_columns!(),
        purchases_and_products!(),
        " WHERE p.item_key = $1 ORDER BY p.bought_at, p.id"
    ))
    .bind(item_key)
    .fetch_all(executor)
    .await?)
}

/// Every category any bought product is in, A to Z.
pub async fn categories<'e, E>(executor: E) -> Result<Vec<String>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT category FROM bought_products ORDER BY category",
    )
    .fetch_all(executor)
    .await?)
}
