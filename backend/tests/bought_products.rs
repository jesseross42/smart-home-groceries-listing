//! Bought products through the real router: saving a shop keeps one record
//! per product, the spending analysis files purchases under that record's
//! category, and Undo brings the record back in step.
//!
//! The stores are the fake catalogue: Woolworths "Full Cream Milk 2L"
//! (`w-milk-2l`, 2L) is $3.10.

mod common;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use common::purchases::{fill_trolley, insert_bought, orders_once_saved, Bought};
use common::trolley::chosen_item;
use common::TestApp;
use grocery_backend::models::db::CategorySource;
use grocery_backend::services::purchases::product_repository::{self, ProductDetails};
use grocery_backend::services::stores::Store;
use serde_json::{json, Value};
use sqlx::PgPool;

/// A July purchase of the same milk the fake Woolworths sells.
const MILK_JULY: Bought = Bought {
    item: "milk",
    store: "woolworths",
    product_id: "w-milk-2l",
    total: "3.10",
    delivery_share: "0",
    category: "Dairy & eggs",
    at: "2026-07-03T02:00:00Z",
};

/// `(product_name, category, count)` of the milk's product record; count is
/// 0 when there is no record.
async fn milk_record(pool: &PgPool) -> (Option<String>, Option<String>, i64) {
    sqlx::query_as(
        "SELECT max(product_name), max(category), count(*) FROM bought_products
         WHERE store = 'woolworths' AND product_id = 'w-milk-2l'",
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Buys two milks through a trolley fill; returns the saved order's id.
async fn buy_milk(app: &TestApp, orders_before: usize) -> String {
    chosen_item(app, "milk", 2, "woolworths", "w-milk-2l").await;
    fill_trolley(
        app,
        json!([{ "product_id": "w-milk-2l", "outcome": "added" }]),
        "0",
    )
    .await;
    let orders = orders_once_saved(app, orders_before + 1).await;
    orders[0]["id"].as_str().unwrap().to_string()
}

async fn spending(app: &TestApp, query: &str) -> Value {
    let (status, body) = app.get(&format!("/api/spending?{query}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test]
async fn a_saved_shop_keeps_a_record_of_each_product(pool: PgPool) {
    let app = TestApp::new(pool.clone());

    buy_milk(&app, 0).await;

    let (name, category, count) = milk_record(&pool).await;
    assert_eq!(count, 1);
    assert_eq!(name.as_deref(), Some("Full Cream Milk"));
    assert_eq!(category.as_deref(), Some("Dairy & eggs"));
}

#[sqlx::test]
async fn spending_files_purchases_under_the_products_category(pool: PgPool) {
    insert_bought(&pool, &MILK_JULY).await;
    sqlx::query("UPDATE bought_products SET category = 'Pantry'")
        .execute(&pool)
        .await
        .unwrap();
    let app = TestApp::new(pool);

    let body = spending(&app, "").await;

    assert_eq!(body["categories"], json!(["Pantry"]));
    assert_eq!(body["by_category"][0]["category"], "Pantry");
    // The total is the purchase's own: the product record never changes it.
    assert_eq!(body["items_total"], "3.10");
    let pantry = spending(&app, "category=Pantry").await;
    assert_eq!(pantry["items_total"], "3.10");
    let dairy = spending(&app, "category=Dairy%20%26%20eggs").await;
    assert_eq!(dairy["items_total"], "0");
}

#[sqlx::test]
async fn undo_of_a_products_only_shop_removes_its_record(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let order = buy_milk(&app, 0).await;

    let (status, _) = app.delete(&format!("/api/purchase-orders/{order}")).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(milk_record(&pool).await.2, 0);
    assert_eq!(spending(&app, "").await["categories"], json!([]));
}

#[sqlx::test]
async fn undo_of_the_newest_shop_refreshes_from_the_older_purchase(pool: PgPool) {
    // The older purchase saved the product id as its name.
    insert_bought(&pool, &MILK_JULY).await;
    let app = TestApp::new(pool.clone());
    let order = buy_milk(&app, 1).await;
    assert_eq!(
        milk_record(&pool).await.0.as_deref(),
        Some("Full Cream Milk")
    );

    let (status, _) = app.delete(&format!("/api/purchase-orders/{order}")).await;

    assert_eq!(status, StatusCode::OK);
    let (name, _, count) = milk_record(&pool).await;
    assert_eq!(count, 1);
    assert_eq!(name.as_deref(), Some("w-milk-2l"));
}

#[sqlx::test]
async fn an_older_purchase_never_overwrites_a_newer_ones_details(pool: PgPool) {
    insert_bought(&pool, &MILK_JULY).await;
    let older: DateTime<Utc> = "2026-06-01T00:00:00Z".parse().unwrap();
    let details = ProductDetails {
        store: Store::Woolworths,
        product_id: "w-milk-2l",
        product_name: "Old Milk Name",
        brand: None,
        package_size: None,
        store_category: None,
        category: "Pantry",
        category_source: CategorySource::Name,
        refreshed_at: older,
    };

    product_repository::upsert_if_newer(&pool, &details)
        .await
        .unwrap();

    let (name, category, _) = milk_record(&pool).await;
    assert_eq!(name.as_deref(), Some("w-milk-2l"));
    assert_eq!(category.as_deref(), Some("Dairy & eggs"));
}
