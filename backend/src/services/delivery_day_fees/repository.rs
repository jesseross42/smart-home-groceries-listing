//! Every SQL statement for `store_delivery_day_fees`.
//!
//! Every statement is a literal `&'static str` with bind parameters.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgExecutor;

use super::DayFee;
use crate::error::ApiError;
use crate::services::stores::Store;

/// Saves one day's cheapest fee, replacing an earlier report.
pub async fn upsert<'e, E>(
    executor: E,
    store: Store,
    date: NaiveDate,
    fee: Decimal,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO store_delivery_day_fees (store, delivery_date, cheapest_fee)
         VALUES ($1, $2, $3)
         ON CONFLICT (store, delivery_date) DO UPDATE SET
             cheapest_fee = EXCLUDED.cheapest_fee,
             reported_at = now()",
    )
    .bind(store)
    .bind(date)
    .bind(fee)
    .execute(executor)
    .await?;
    Ok(())
}

/// Every store's fees on `today` or later, earliest first.
pub async fn from<'e, E>(executor: E, today: NaiveDate) -> Result<Vec<DayFee>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, DayFee>(
        "SELECT store, delivery_date, cheapest_fee
         FROM store_delivery_day_fees
         WHERE delivery_date >= $1
         ORDER BY delivery_date, store",
    )
    .bind(today)
    .fetch_all(executor)
    .await?)
}
