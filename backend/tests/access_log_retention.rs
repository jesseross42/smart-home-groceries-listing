//! Retention for `access_logs`: what the daily clean-up deletes
//! (`docs/features/FEATURE_ACCESS_LOGS.md`).

use grocery_backend::services::access_log::retention;
use sqlx::PgPool;

/// Stores one request that happened `days_ago` days ago, at `path`.
async fn log_request(pool: &PgPool, path: &str, days_ago: i32) {
    sqlx::query(
        "INSERT INTO access_logs (occurred_at, user_id, method, path, status_code, duration_ms)
         VALUES (now() - make_interval(days => $1), 'dev-user', 'GET', $2, 200, 1)",
    )
    .bind(days_ago)
    .bind(path)
    .execute(pool)
    .await
    .expect("storing an access log row");
}

/// The stored paths, oldest first.
async fn stored_paths(pool: &PgPool) -> Vec<String> {
    sqlx::query_scalar::<_, String>("SELECT path FROM access_logs ORDER BY occurred_at")
        .fetch_all(pool)
        .await
        .expect("reading access_logs")
}

#[sqlx::test]
async fn rows_older_than_the_limit_are_deleted(pool: PgPool) {
    log_request(&pool, "/old", 31).await;
    log_request(&pool, "/kept", 29).await;
    log_request(&pool, "/new", 0).await;

    let deleted = retention::purge(&pool, 30).await.unwrap();

    assert_eq!(deleted, 1);
    assert_eq!(stored_paths(&pool).await, ["/kept", "/new"]);
}

#[sqlx::test]
async fn zero_days_keeps_everything(pool: PgPool) {
    log_request(&pool, "/ancient", 5000).await;

    let deleted = retention::purge(&pool, 0).await.unwrap();

    assert_eq!(deleted, 0);
    assert_eq!(stored_paths(&pool).await, ["/ancient"]);
}

#[sqlx::test]
async fn a_limit_too_large_for_the_database_deletes_nothing(pool: PgPool) {
    log_request(&pool, "/old", 400).await;

    let deleted = retention::purge(&pool, u32::MAX).await;

    // Either nothing is old enough, or PostgreSQL refuses the date; never a
    // wrong deletion.
    assert!(matches!(deleted, Ok(0) | Err(_)));
    assert_eq!(stored_paths(&pool).await, ["/old"]);
}
