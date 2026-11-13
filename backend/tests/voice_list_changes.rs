//! The web app's view of voice changes: `GET /api/voice-list-changes` and
//! `POST /api/voice-list-changes/{id}/undo`.

mod common;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::{TestApp, TEST_BRIDGE_SECRET};
use grocery_backend::routes::alexa::BRIDGE_SECRET_HEADER;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

/// "Alexa, remove …" as the bridge sidecar; returns the change's id.
async fn remove_by_voice(app: &TestApp, body: &Value) -> String {
    let (status, change) = app
        .send(
            Request::builder()
                .method(Method::POST)
                .uri("/api/intake/alexa/remove")
                .header("content-type", "application/json")
                .header(BRIDGE_SECRET_HEADER, TEST_BRIDGE_SECRET)
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "{change}");
    change["id"].as_str().unwrap().to_string()
}

async fn undo(app: &TestApp, change_id: &str) -> (StatusCode, Value) {
    app.post(&format!("/api/voice-list-changes/{change_id}/undo"))
        .await
}

/// The quantity of the one item named `name`, or `None` when it is gone.
async fn quantity_of(app: &TestApp, name: &str) -> Option<i64> {
    let (_, items) = app.get("/api/grocery-items").await;
    items
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["name"] == name)
        .map(|item| item["quantity"].as_i64().unwrap())
}

/// Makes a change look `minutes` old.
async fn age_change(pool: &PgPool, change_id: &str, minutes: i32) {
    sqlx::query(
        "UPDATE voice_list_changes SET created_at = now() - make_interval(mins => $2)
         WHERE id = $1",
    )
    .bind(Uuid::parse_str(change_id).unwrap())
    .bind(minutes)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test]
async fn the_view_needs_a_session(pool: PgPool) {
    let app = TestApp::with_auth(pool);

    let (listed, _) = app.get("/api/voice-list-changes").await;
    let (undone, _) = undo(&app, &Uuid::new_v4().to_string()).await;

    assert_eq!(listed, StatusCode::UNAUTHORIZED);
    assert_eq!(undone, StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn changes_are_listed_newest_first_with_their_quantities(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    app.add_item(&json!({ "name": "milk", "quantity": 3 }))
        .await;
    app.add_item(&json!({ "name": "bread", "quantity": 1 }))
        .await;
    let reduced = remove_by_voice(&app, &json!({ "item": "milk", "quantity": 2 })).await;
    // Keeps the order certain even on a clock with coarse steps.
    age_change(&pool, &reduced, 1).await;
    remove_by_voice(&app, &json!({ "item": "bread" })).await;

    let (status, changes) = app.get("/api/voice-list-changes").await;

    assert_eq!(status, StatusCode::OK);
    let changes = changes.as_array().unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0]["item_name"], "bread");
    assert_eq!(changes[0]["kind"], "removed");
    assert_eq!(changes[0]["quantity_after"], 0);
    assert_eq!(changes[1]["item_name"], "milk");
    assert_eq!(changes[1]["kind"], "reduced");
    assert_eq!(changes[1]["quantity_before"], 3);
    assert_eq!(changes[1]["quantity_after"], 1);
    assert_eq!(changes[1]["undone_at"], Value::Null);
    assert_eq!(changes[1]["can_undo"], true);
    assert!(changes[1]["created_at"].is_string());
}

#[sqlx::test]
async fn undo_puts_a_removed_item_back(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "quantity": 2 }))
        .await;
    let change = remove_by_voice(&app, &json!({ "item": "milk" })).await;
    assert_eq!(quantity_of(&app, "milk").await, None);

    let (status, body) = undo(&app, &change).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(body["undone_at"].is_string());
    assert_eq!(body["can_undo"], false);
    assert_eq!(quantity_of(&app, "milk").await, Some(2));
}

#[sqlx::test]
async fn undo_can_pick_an_older_change_and_leaves_the_newer_one(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "quantity": 3 }))
        .await;
    app.add_item(&json!({ "name": "bread", "quantity": 1 }))
        .await;
    let older = remove_by_voice(&app, &json!({ "item": "milk", "quantity": 2 })).await;
    remove_by_voice(&app, &json!({ "item": "bread" })).await;

    let (status, _) = undo(&app, &older).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(quantity_of(&app, "milk").await, Some(3));
    assert_eq!(quantity_of(&app, "bread").await, None);
}

#[sqlx::test]
async fn a_change_cannot_be_undone_twice(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "quantity": 3 }))
        .await;
    let change = remove_by_voice(&app, &json!({ "item": "milk", "quantity": 1 })).await;
    undo(&app, &change).await;

    let (status, _) = undo(&app, &change).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(quantity_of(&app, "milk").await, Some(3));
}

#[sqlx::test]
async fn a_change_older_than_the_window_cannot_be_undone(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    app.add_item(&json!({ "name": "milk", "quantity": 2 }))
        .await;
    let change = remove_by_voice(&app, &json!({ "item": "milk" })).await;
    age_change(&pool, &change, 31).await;

    let (listed, changes) = app.get("/api/voice-list-changes").await;
    let (status, _) = undo(&app, &change).await;

    assert_eq!(listed, StatusCode::OK);
    assert_eq!(changes[0]["can_undo"], false);
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(quantity_of(&app, "milk").await, None);
}

#[sqlx::test]
async fn a_reduced_item_committed_since_cannot_be_changed_back(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "quantity": 3 }))
        .await;
    let change = remove_by_voice(&app, &json!({ "item": "milk", "quantity": 1 })).await;
    app.post("/api/grocery-items/commit").await;

    let (status, _) = undo(&app, &change).await;
    let (_, changes) = app.get("/api/voice-list-changes").await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(quantity_of(&app, "milk").await, Some(2));
    assert_eq!(changes[0]["undone_at"], Value::Null);
}

#[sqlx::test]
async fn an_unknown_change_is_not_found(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = undo(&app, &Uuid::new_v4().to_string()).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
