//! Every SQL statement the access log needs. Only `access_logs` is touched,
//! and every statement is a literal with bind parameters.

use sqlx::PgPool;

use super::{AccessLogEntry, AccessLogFilter};
use crate::error::ApiError;
use crate::models::access_log_rows::AccessLogRow;

/// Stores one answered request.
pub async fn insert(pool: &PgPool, entry: &AccessLogEntry) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO access_logs
             (source_ip, forwarded_for, user_id, method, path, status_code, duration_ms)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(entry.source_ip.as_deref())
    .bind(entry.forwarded_for.as_deref())
    .bind(&entry.user_id)
    .bind(&entry.method)
    .bind(&entry.path)
    .bind(i16::try_from(entry.status_code).unwrap_or(i16::MAX))
    .bind(i32::try_from(entry.duration_ms).unwrap_or(i32::MAX))
    .execute(pool)
    .await?;
    Ok(())
}

/// Up to `limit` requests, newest first, older than `before` when given.
/// `filter` leaves out the health probe and keeps one user or one path
/// prefix, each only when asked.
pub async fn list_page(
    pool: &PgPool,
    before: Option<i64>,
    limit: i64,
    filter: &AccessLogFilter,
) -> Result<Vec<AccessLogRow>, ApiError> {
    Ok(sqlx::query_as::<_, AccessLogRow>(
        "SELECT id, occurred_at, source_ip, forwarded_for, user_id, method, path,
                status_code, duration_ms
         FROM access_logs
         WHERE ($1::bigint IS NULL OR id < $1)
           AND ($3::text IS NULL OR path <> $3)
           AND ($4::text IS NULL OR user_id = $4)
           AND ($5::text IS NULL OR starts_with(path, $5))
         ORDER BY id DESC
         LIMIT $2",
    )
    .bind(before)
    .bind(limit)
    .bind(filter.skip_path())
    .bind(filter.user())
    .bind(filter.path_prefix())
    .fetch_all(pool)
    .await?)
}

/// Deletes every request stored more than `days` days ago; returns how many.
pub async fn delete_older_than(pool: &PgPool, days: i32) -> Result<u64, sqlx::Error> {
    let deleted = sqlx::query(
        "DELETE FROM access_logs
         WHERE occurred_at < now() - make_interval(days => $1)",
    )
    .bind(days)
    .execute(pool)
    .await?;
    Ok(deleted.rows_affected())
}
