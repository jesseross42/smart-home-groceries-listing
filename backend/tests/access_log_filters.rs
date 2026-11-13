//! The `/settings/logs` filters on `GET /api/access-logs`: by user and by
//! the start of the path (`docs/features/FEATURE_ACCESS_LOGS.md`).

mod common;
// Not in `common/mod.rs`: only the access-log tests read these rows.
#[path = "common/access_logs.rs"]
mod logged;

use axum::http::StatusCode;
use common::TestApp;
use logged::wait_for_logged;
use serde_json::Value;
use sqlx::PgPool;

/// The paths on one page, newest first.
fn paths(page: &Value) -> Vec<&str> {
    page["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|e| e["path"].as_str().expect("path"))
        .collect()
}

/// Logs one signed-in list read, one signed-in rules read and one webhook
/// refusal with no session, in that order.
async fn log_three_requests(app: &TestApp, pool: &PgPool) {
    app.get("/api/grocery-items").await;
    wait_for_logged(pool, 1).await;
    app.get("/api/item-rules").await;
    wait_for_logged(pool, 2).await;
    app.post_webhook_unauthenticated(&serde_json::json!({ "item": "milk" }))
        .await;
    wait_for_logged(pool, 3).await;
}

#[sqlx::test]
async fn the_user_filter_keeps_only_that_user(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    log_three_requests(&app, &pool).await;

    let (status, page) = app.get("/api/access-logs?user=unauthenticated").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(paths(&page), ["/api/voice-requests"]);
}

#[sqlx::test]
async fn the_path_filter_keeps_paths_that_start_with_it(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    log_three_requests(&app, &pool).await;

    let (_, page) = app.get("/api/access-logs?path=/api/item").await;

    assert_eq!(paths(&page), ["/api/item-rules"]);
}

#[sqlx::test]
async fn both_filters_apply_together(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    log_three_requests(&app, &pool).await;

    let (_, page) = app
        .get("/api/access-logs?user=dev-user&path=/api/voice")
        .await;

    assert!(paths(&page).is_empty());
}

#[sqlx::test]
async fn a_path_filter_is_not_a_pattern(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    log_three_requests(&app, &pool).await;

    let (status, page) = app.get("/api/access-logs?path=%25").await;

    assert_eq!(status, StatusCode::OK);
    assert!(paths(&page).is_empty());
}

#[sqlx::test]
async fn blank_filters_list_everything(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    log_three_requests(&app, &pool).await;

    let (_, page) = app.get("/api/access-logs?user=&path=%20").await;

    // The three requests; reading the log is itself logged only afterwards.
    assert!(paths(&page).len() >= 3);
}

#[sqlx::test]
async fn filtered_pages_keep_paging(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    for _ in 0..3 {
        app.get("/api/grocery-items").await;
    }
    app.get("/api/item-rules").await;
    wait_for_logged(&pool, 4).await;

    let (_, first) = app.get("/api/access-logs?path=/api/grocery&limit=2").await;
    let before = first["next_before"].as_i64().expect("a full page");
    let (_, second) = app
        .get(&format!(
            "/api/access-logs?path=/api/grocery&limit=2&before={before}"
        ))
        .await;

    assert_eq!(paths(&first).len(), 2);
    assert_eq!(paths(&second), ["/api/grocery-items"]);
}
