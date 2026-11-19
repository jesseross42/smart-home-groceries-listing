//! Every SQL statement on `bought_products`: one row per store product the
//! household has bought (migration `20261119090000`). The rules for when a
//! row changes are in [`super::products`].
//!
//! Every statement is a literal `&'static str` with bind parameters.

use chrono::{DateTime, Utc};
use sqlx::PgExecutor;

use crate::error::ApiError;
use crate::models::db::CategorySource;
use crate::services::stores::Store;

/// A product's details, as one purchase of it gave them, and its category.
#[derive(Debug)]
pub struct ProductDetails<'a> {
    pub store: Store,
    pub product_id: &'a str,
    pub product_name: &'a str,
    pub brand: Option<&'a str>,
    pub package_size: Option<&'a str>,
    pub store_category: Option<&'a str>,
    pub category: &'a str,
    pub category_source: CategorySource,
    /// When the purchase the details came from was bought.
    pub refreshed_at: DateTime<Utc>,
}

/// The newest purchase of one product: what refreshing its row reads.
#[derive(Debug, sqlx::FromRow)]
pub struct NewestPurchase {
    pub product_name: String,
    pub brand: Option<String>,
    pub package_size: Option<String>,
    pub store_category: Option<String>,
    pub bought_at: DateTime<Utc>,
}

/// One product's key and the texts its category is worked out from.
#[derive(Debug, sqlx::FromRow)]
pub struct CategoryInput {
    pub store: Store,
    pub product_id: String,
    pub store_category: Option<String>,
    pub product_name: String,
}

/// Writes `details` unless the row already holds a newer purchase's.
pub async fn upsert_if_newer<'e, E>(
    executor: E,
    details: &ProductDetails<'_>,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO bought_products
             (store, product_id, product_name, brand, package_size, store_category,
              category, category_source, refreshed_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
         ON CONFLICT (store, product_id) DO UPDATE SET
             product_name = EXCLUDED.product_name,
             brand = EXCLUDED.brand,
             package_size = EXCLUDED.package_size,
             store_category = EXCLUDED.store_category,
             category = EXCLUDED.category,
             category_source = EXCLUDED.category_source,
             refreshed_at = EXCLUDED.refreshed_at
         WHERE bought_products.refreshed_at <= EXCLUDED.refreshed_at",
    )
    .bind(details.store)
    .bind(details.product_id)
    .bind(details.product_name)
    .bind(details.brand)
    .bind(details.package_size)
    .bind(details.store_category)
    .bind(details.category)
    .bind(details.category_source)
    .bind(details.refreshed_at)
    .execute(executor)
    .await?;
    Ok(())
}

/// Overwrites an existing row with `details`, older or not (Undo).
pub async fn overwrite<'e, E>(executor: E, details: &ProductDetails<'_>) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "UPDATE bought_products SET
             product_name = $3, brand = $4, package_size = $5, store_category = $6,
             category = $7, category_source = $8, refreshed_at = $9
         WHERE store = $1 AND product_id = $2",
    )
    .bind(details.store)
    .bind(details.product_id)
    .bind(details.product_name)
    .bind(details.brand)
    .bind(details.package_size)
    .bind(details.store_category)
    .bind(details.category)
    .bind(details.category_source)
    .bind(details.refreshed_at)
    .execute(executor)
    .await?;
    Ok(())
}

/// The newest purchase of one product, if any is left.
pub async fn newest_purchase<'e, E>(
    executor: E,
    store: Store,
    product_id: &str,
) -> Result<Option<NewestPurchase>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, NewestPurchase>(
        "SELECT product_name, brand, package_size, store_category, bought_at
         FROM purchases
         WHERE store = $1 AND product_id = $2
         ORDER BY bought_at DESC, created_at DESC, id DESC
         LIMIT 1",
    )
    .bind(store)
    .bind(product_id)
    .fetch_optional(executor)
    .await?)
}

/// Deletes one product's row. Only called once no purchase of it is left.
pub async fn delete<'e, E>(executor: E, store: Store, product_id: &str) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query("DELETE FROM bought_products WHERE store = $1 AND product_id = $2")
        .bind(store)
        .bind(product_id)
        .execute(executor)
        .await?;
    Ok(())
}

/// Every product's key and category texts.
pub async fn category_inputs<'e, E>(executor: E) -> Result<Vec<CategoryInput>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, CategoryInput>(
        "SELECT store, product_id, store_category, product_name FROM bought_products",
    )
    .fetch_all(executor)
    .await?)
}

/// Sets one product's category; `true` when it changed.
pub async fn set_category<'e, E>(
    executor: E,
    store: Store,
    product_id: &str,
    category: &str,
    source: CategorySource,
) -> Result<bool, ApiError>
where
    E: PgExecutor<'e>,
{
    let result = sqlx::query(
        "UPDATE bought_products SET category = $3, category_source = $4
         WHERE store = $1 AND product_id = $2
           AND (category <> $3 OR category_source <> $4)",
    )
    .bind(store)
    .bind(product_id)
    .bind(category)
    .bind(source)
    .execute(executor)
    .await?;
    Ok(result.rows_affected() > 0)
}
