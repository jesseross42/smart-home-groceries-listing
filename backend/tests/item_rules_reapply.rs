//! Putting a rule's chips on items already on the list:
//! `POST /api/item-rules/{id}/apply`.
//!
//! Rules normally act only as an item arrives (`tests/item_rules_applied.rs`).
//! This is the household asking for a new or edited rule to reach the items
//! that were there first. The reply carries each changed item's previous
//! chips, which is what Undo writes back.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

/// Stores a voice-only rule and returns its id.
async fn rule(app: &TestApp, triggers: &[&str], terms: &[&str]) -> String {
    let (status, created) = app
        .post_json(
            "/api/item-rules",
            &json!({ "triggers": triggers, "filter_terms": terms }),
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "setup failed: {created}");
    created["id"].as_str().unwrap().to_string()
}

/// Applies a rule to the list, failing the test unless that is a 200.
async fn apply(app: &TestApp, rule_id: &str) -> Value {
    let (status, body) = app.post(&format!("/api/item-rules/{rule_id}/apply")).await;
    assert_eq!(status, StatusCode::OK, "apply failed: {body}");
    body
}

/// The chips now stored on one list item.
async fn stored_terms(app: &TestApp, item_id: &str) -> Value {
    let (_, list) = app.get("/api/grocery-items").await;
    list.as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == item_id)
        .expect("item is on the list")["filter_terms"]
        .clone()
}

#[sqlx::test]
async fn a_matching_item_already_on_the_list_gets_the_chips(pool: PgPool) {
    let app = TestApp::new(pool);
    let item_id = app.add_item(&json!({ "name": "Toilet Paper" })).await;
    let rule_id = rule(&app, &["toilet paper"], &["3 ply"]).await;

    let body = apply(&app, &rule_id).await;

    assert_eq!(body["changed"].as_array().unwrap().len(), 1);
    assert_eq!(body["changed"][0]["id"], item_id);
    assert_eq!(body["changed"][0]["name"], "Toilet Paper");
    assert_eq!(body["changed"][0]["filter_terms"], json!(["3 ply"]));
    assert_eq!(body["changed"][0]["previous_filter_terms"], json!([]));
    assert_eq!(stored_terms(&app, &item_id).await, json!(["3 ply"]));
}

#[sqlx::test]
async fn an_items_own_chips_stay_first(pool: PgPool) {
    let app = TestApp::new(pool);
    let _item_id = app
        .add_item(&json!({ "name": "milk", "filter_terms": ["2L"] }))
        .await;
    let rule_id = rule(&app, &["milk"], &["lactose free"]).await;

    let body = apply(&app, &rule_id).await;

    assert_eq!(
        body["changed"][0]["filter_terms"],
        json!(["2L", "lactose free"])
    );
    assert_eq!(body["changed"][0]["previous_filter_terms"], json!(["2L"]));
}

#[sqlx::test]
async fn items_that_do_not_match_or_already_have_the_chips_are_left_alone(pool: PgPool) {
    let app = TestApp::new(pool);
    let bread = app.add_item(&json!({ "name": "bread" })).await;
    let chipped = app
        .add_item(&json!({ "name": "milk", "filter_terms": ["Lactose Free"] }))
        .await;
    let rule_id = rule(&app, &["milk"], &["lactose free"]).await;

    let body = apply(&app, &rule_id).await;

    assert_eq!(body["changed"], json!([]));
    assert_eq!(stored_terms(&app, &bread).await, json!([]));
    assert_eq!(stored_terms(&app, &chipped).await, json!(["Lactose Free"]));
}

#[sqlx::test]
async fn a_committed_item_is_left_alone(pool: PgPool) {
    let app = TestApp::new(pool);
    let item_id = app.add_item(&json!({ "name": "milk" })).await;
    let (status, _) = app.post("/api/grocery-items/commit").await;
    assert_eq!(status, StatusCode::OK);
    let rule_id = rule(&app, &["milk"], &["lactose free"]).await;

    let body = apply(&app, &rule_id).await;

    assert_eq!(body["changed"], json!([]));
    assert_eq!(stored_terms(&app, &item_id).await, json!([]));
}

#[sqlx::test]
async fn previous_chips_put_back_by_an_item_edit_undo_the_apply(pool: PgPool) {
    let app = TestApp::new(pool);
    let item_id = app
        .add_item(&json!({ "name": "milk", "filter_terms": ["2L"] }))
        .await;
    let rule_id = rule(&app, &["milk"], &["lactose free"]).await;
    let body = apply(&app, &rule_id).await;

    let (status, _) = app
        .patch_json(
            &format!("/api/grocery-items/{item_id}"),
            &json!({ "filter_terms": body["changed"][0]["previous_filter_terms"] }),
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(stored_terms(&app, &item_id).await, json!(["2L"]));
}

#[sqlx::test]
async fn an_unknown_rule_is_a_404(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app
        .post(&format!("/api/item-rules/{}/apply", Uuid::new_v4()))
        .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}
