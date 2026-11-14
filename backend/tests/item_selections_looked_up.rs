//! Choosing a product looked up by its link, through the real router.
//!
//! `PUT /api/grocery-items/{id}/selection` with `looked_up: true` reads the
//! product from the store's product page (`StoreClient::product`), so it
//! need not be in the item's search. The stores are the fake catalogue.

mod common;

use axum::http::StatusCode;
use common::TestApp;
use serde_json::json;
use sqlx::PgPool;

fn uri(item_id: &str) -> String {
    format!("/api/grocery-items/{item_id}/selection")
}

#[sqlx::test]
async fn a_looked_up_product_outside_the_search_is_chosen_at_the_items_quantity(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "breakfast", "quantity": 2 }))
        .await;

    let (status, body) = app
        .put_json(
            &uri(&id),
            &json!({ "store": "coles", "product_id": "c-oat-1l", "looked_up": true }),
        )
        .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["product_id"], "c-oat-1l");
    // Priced from the store's own answer, for the item's quantity.
    assert_eq!(body["price"], "3.90");
    assert_eq!(body["total_price"], "6.50");
    assert_eq!(body["priced_quantity"], 2);
}

#[sqlx::test]
async fn without_looked_up_a_product_outside_the_search_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "breakfast" })).await;

    let (status, body) = app
        .put_json(
            &uri(&id),
            &json!({ "store": "coles", "product_id": "c-oat-1l" }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
}

#[sqlx::test]
async fn a_looked_up_product_the_store_does_not_sell_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app.add_item(&json!({ "name": "milk" })).await;

    let (status, body) = app
        .put_json(
            &uri(&id),
            &json!({ "store": "coles", "product_id": "w-milk-2l", "looked_up": true }),
        )
        .await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    assert!(
        body["detail"].as_str().unwrap().contains("does not sell"),
        "{body}"
    );
}
