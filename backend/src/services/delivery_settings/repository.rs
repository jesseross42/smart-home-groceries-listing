//! Every SQL statement for `store_delivery_fees` and `order_preferences`.
//!
//! Every statement is a literal `&'static str` with bind parameters.

use sqlx::PgExecutor;

use super::{FeeRules, Preferences};
use crate::error::ApiError;
use crate::models::delivery_rows::{OrderPreferences, StoreDeliveryFees};

/// Every store's saved fee rules. A store never saved has no row.
pub async fn list_fees<'e, E>(executor: E) -> Result<Vec<StoreDeliveryFees>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, StoreDeliveryFees>(
        "SELECT store, delivery_fee, free_delivery_over, minimum_order, delivery_address,
                updated_by, updated_at
         FROM store_delivery_fees",
    )
    .fetch_all(executor)
    .await?)
}

/// Saves one store's fee rules, replacing what was there.
pub async fn upsert_fees<'e, E>(
    executor: E,
    rules: &FeeRules,
    user_id: &str,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO store_delivery_fees
             (store, delivery_fee, free_delivery_over, minimum_order, delivery_address,
              updated_by)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (store) DO UPDATE SET
             delivery_fee = EXCLUDED.delivery_fee,
             free_delivery_over = EXCLUDED.free_delivery_over,
             minimum_order = EXCLUDED.minimum_order,
             delivery_address = EXCLUDED.delivery_address,
             updated_by = EXCLUDED.updated_by,
             updated_at = now()",
    )
    .bind(rules.store)
    .bind(rules.delivery_fee)
    .bind(rules.free_delivery_over)
    .bind(rules.minimum_order)
    .bind(rules.delivery_address.as_deref())
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(())
}

/// The planner's saved preferences, if they were ever saved.
pub async fn find_preferences<'e, E>(executor: E) -> Result<Option<OrderPreferences>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, OrderPreferences>(
        "SELECT mode, max_delivery_spend, default_delivery_day, allowed_delivery_days, need_by, perishable_hours,
                updated_by, updated_at
         FROM order_preferences WHERE id = 1",
    )
    .fetch_optional(executor)
    .await?)
}

/// Saves the planner's preferences, replacing what was there.
pub async fn upsert_preferences<'e, E>(
    executor: E,
    preferences: &Preferences,
    user_id: &str,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO order_preferences
             (id, mode, max_delivery_spend, default_delivery_day, allowed_delivery_days, need_by, perishable_hours,
              updated_by)
         VALUES (1, $1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (id) DO UPDATE SET
             mode = EXCLUDED.mode,
             max_delivery_spend = EXCLUDED.max_delivery_spend,
             default_delivery_day = EXCLUDED.default_delivery_day,
             allowed_delivery_days = EXCLUDED.allowed_delivery_days,
             need_by = EXCLUDED.need_by,
             perishable_hours = EXCLUDED.perishable_hours,
             updated_by = EXCLUDED.updated_by,
             updated_at = now()",
    )
    .bind(preferences.mode)
    .bind(preferences.max_delivery_spend)
    .bind(preferences.default_delivery_day)
    .bind(&preferences.allowed_delivery_days)
    .bind(preferences.need_by)
    .bind(preferences.perishable_hours)
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(())
}
