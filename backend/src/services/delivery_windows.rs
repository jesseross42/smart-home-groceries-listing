//! The latest delivery windows reported from each store's logged-in tab.

use chrono::{DateTime, NaiveDateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use sqlx::PgPool;

use crate::error::ApiError;
use crate::services::stores::Store;

/// One available regular delivery window in store-local time.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
pub struct DeliveryWindow {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
    pub fee: Decimal,
}

/// One store's complete snapshot of the dates visible in its regular picker.
#[derive(sqlx::FromRow)]
pub struct WindowSnapshot {
    pub store: Store,
    pub windows: Json<Vec<DeliveryWindow>>,
}

/// Replaces a snapshot, including when the store reported zero windows.
pub async fn replace(
    pool: &PgPool,
    store: Store,
    windows: Vec<DeliveryWindow>,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO store_delivery_window_snapshots (store, windows)
         VALUES ($1, $2)
         ON CONFLICT (store) DO UPDATE SET windows = EXCLUDED.windows, reported_at = now()",
    )
    .bind(store)
    .bind(Json(windows))
    .execute(pool)
    .await?;
    Ok(())
}

/// The 15-minute snapshots still fresh enough for planning.
pub async fn fresh(pool: &PgPool, now: DateTime<Utc>) -> Result<Vec<WindowSnapshot>, ApiError> {
    Ok(sqlx::query_as(
        "SELECT store, windows FROM store_delivery_window_snapshots
         WHERE reported_at > $1::timestamptz - interval '15 minutes'",
    )
    .bind(now)
    .fetch_all(pool)
    .await?)
}
