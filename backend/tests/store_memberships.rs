//! Settings › Member prices through the real router: `GET` and
//! `PUT /api/store-memberships`, and what membership does to a search.
//!
//! The fake Woolworths 3L milk ($5.40) has a member price of $4.90.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

const URI: &str = "/api/store-memberships";

async fn set_member(app: &TestApp, member: bool) -> Value {
    let (status, body) = app
        .put_json(
            URI,
            &json!({ "stores": [{ "store": "woolworths", "member": member }] }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// The fake Woolworths 3L milk, as a search for milk shows it.
async fn three_litre_milk(app: &TestApp, item_id: &str) -> Value {
    let (status, body) = app
        .get(&format!("/api/grocery-items/{item_id}/products"))
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["stores"][0]["products"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["product_id"] == "w-milk-3l")
        .cloned()
        .unwrap()
}

#[sqlx::test]
async fn only_woolworths_is_listed_and_nobody_is_a_member_at_first(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, body) = app.get(URI).await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(
        body,
        json!({ "stores": [{
            "store": "woolworths", "store_name": "Woolworths",
            "program": "Everyday Rewards", "member": false,
        }] })
    );
}

#[sqlx::test]
async fn a_saved_membership_is_read_back_and_can_be_undone(pool: PgPool) {
    let app = TestApp::new(pool);

    let saved = set_member(&app, true).await;
    let (_, read) = app.get(URI).await;
    assert_eq!(read, saved);
    assert_eq!(read["stores"][0]["member"], true);

    set_member(&app, false).await;
    let (_, read) = app.get(URI).await;
    assert_eq!(read["stores"][0]["member"], false);
}

#[sqlx::test]
async fn coles_has_no_member_prices_to_save(pool: PgPool) {
    let app = TestApp::new(pool);

    let (status, _) = app
        .put_json(
            URI,
            &json!({ "stores": [{ "store": "coles", "member": true }] }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn an_empty_or_unknown_store_list_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);

    let (empty, _) = app.put_json(URI, &json!({ "stores": [] })).await;
    let (unknown, _) = app
        .put_json(
            URI,
            &json!({ "stores": [{ "store": "aldi", "member": true }] }),
        )
        .await;

    assert_eq!(empty, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(unknown, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn a_non_member_sees_the_shelf_price_only(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "Milk" })).await;

    let milk = three_litre_milk(&app, &id).await;

    assert_eq!(milk["price"], "5.40");
    assert_eq!(milk["member_price"], false);
}

#[sqlx::test]
async fn a_member_pays_the_member_price_at_once(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "Milk" })).await;
    three_litre_milk(&app, &id).await; // fills the search cache

    set_member(&app, true).await;
    let milk = three_litre_milk(&app, &id).await;

    assert_eq!(milk["price"], "4.90");
    assert_eq!(milk["was_price"], "6.20");
    assert_eq!(milk["member_price"], true);
    assert_eq!(milk["total_price"], "4.90");
}
