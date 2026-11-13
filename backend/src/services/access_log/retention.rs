//! Retention for the `access_logs` table: rows older than
//! `ACCESS_LOG_RETENTION_DAYS` are deleted once at startup and then daily.
//! The log files have their own limit (`crate::logging`).

use std::time::Duration;

use sqlx::PgPool;

use super::repository;

/// How often the clean-up runs.
const PURGE_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// Starts the daily clean-up for the life of the process. `days == 0` keeps
/// every row, so nothing is started.
pub fn spawn(pool: PgPool, days: u32) {
    if days == 0 {
        return;
    }
    tokio::spawn(async move {
        loop {
            match purge(&pool, days).await {
                Ok(deleted) => tracing::info!(deleted, days, "old access log rows deleted"),
                // Tried again tomorrow; the table only grows a little meanwhile.
                Err(err) => tracing::warn!(%err, "could not delete old access log rows"),
            }
            tokio::time::sleep(PURGE_EVERY).await;
        }
    });
}

/// Deletes rows stored more than `days` days ago and returns how many went.
/// `days == 0` deletes nothing.
pub async fn purge(pool: &PgPool, days: u32) -> Result<u64, sqlx::Error> {
    if days == 0 {
        return Ok(0);
    }
    repository::delete_older_than(pool, i32::try_from(days).unwrap_or(i32::MAX)).await
}
