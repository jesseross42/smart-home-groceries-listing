//! The order planner and the household's dislikes, through the real router.
//!
//! The stores are the fake catalogue (`STORE_CLIENTS=fake`): milk is $3.10
//! (2L) at Woolworths and $4.95 (3L) at Coles. Delivery is free here, so
//! only the dislike decides between the stores.

mod common;

use axum::http::StatusCode;
use common::trolley::chosen_item;
use common::TestApp;
use serde_json::{json, Value};
use sqlx::PgPool;

async fn put_ok(app: &TestApp, path: &str, body: Value) {
    let (status, answer) = app.put_json(path, &body).await;
    assert_eq!(status, StatusCode::OK, "{answer}");
}

/// Milk chosen at both stores, committed, with free delivery at both.
async fn milk_at_both_stores(app: &TestApp) -> String {
    let milk = chosen_item(app, "milk", 1, "woolworths", "w-milk-2l").await;
    put_ok(
        app,
        &format!("/api/grocery-items/{milk}/selection"),
        json!({ "store": "coles", "product_id": "c-milk-3l" }),
    )
    .await;
    app.post("/api/grocery-items/commit").await;
    let free = |store| json!({ "store": store, "delivery_fee": "0" });
    put_ok(
        app,
        "/api/delivery-settings",
        json!({ "stores": [free("woolworths"), free("coles")], "mode": "minimise_total" }),
    )
    .await;
    milk
}

async fn dislike(app: &TestApp, store: &str, product_id: &str) {
    put_ok(
        app,
        "/api/product-dislikes",
        json!({ "store": store, "product_id": product_id, "name": "Full Cream Milk" }),
    )
    .await;
}

async fn options(app: &TestApp) -> Value {
    let (status, body) = app.get("/api/order-options").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test]
async fn the_planner_skips_a_disliked_product(pool: PgPool) {
    let app = TestApp::new(pool);
    milk_at_both_stores(&app).await;
    dislike(&app, "woolworths", "w-milk-2l").await;

    let body = options(&app).await;

    let best = &body["options"][0];
    assert_eq!(best["kind"], "coles", "{body}");
    assert_eq!(best["recommended"], true);
    assert_eq!(best["disliked"], json!([]));
    let woolworths = &body["options"][1];
    assert_eq!(woolworths["kind"], "woolworths", "{body}");
    assert_eq!(woolworths["disliked"][0]["name"], "milk");
    assert_eq!(woolworths["disliked"][0]["store_name"], "Woolworths");
    assert_eq!(woolworths["disliked"][0]["disliked_by"], json!(["dev"]));
    assert_eq!(woolworths["disliked"][0]["avoidable"], true);
}

#[sqlx::test]
async fn buy_it_this_time_lets_the_planner_pick_the_product_again(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = milk_at_both_stores(&app).await;
    dislike(&app, "woolworths", "w-milk-2l").await;
    put_ok(
        &app,
        &format!("/api/grocery-items/{milk}/dislike-override"),
        json!({ "store": "woolworths", "product_id": "w-milk-2l" }),
    )
    .await;

    let body = options(&app).await;

    assert_eq!(body["options"][0]["kind"], "woolworths", "{body}");
    assert_eq!(body["options"][0]["disliked"], json!([]));
}

#[sqlx::test]
async fn when_every_product_is_disliked_the_cheapest_is_kept_and_named(pool: PgPool) {
    let app = TestApp::new(pool);
    milk_at_both_stores(&app).await;
    dislike(&app, "woolworths", "w-milk-2l").await;
    dislike(&app, "coles", "c-milk-3l").await;

    let body = options(&app).await;

    let best = &body["options"][0];
    assert_eq!(best["kind"], "woolworths", "{body}");
    assert_eq!(best["recommended"], true);
    assert_eq!(best["disliked"][0]["disliked_by"], json!(["dev"]));
    assert_eq!(best["disliked"][0]["avoidable"], false);
}
