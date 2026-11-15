//! New items get their cheapest-per-unit products chosen automatically
//! (`AUTO_PICK_PRODUCTS`), through the real router and the fake catalogue.

mod common;

use axum::http::StatusCode;
use common::{fake_store_settings, test_settings, TestApp};
use grocery_backend::config::{Settings, StoreSettings};
use serde_json::{json, Value};
use sqlx::PgPool;

fn auto_pick_app(pool: PgPool) -> TestApp {
    TestApp::with_settings(
        pool,
        Settings {
            stores: StoreSettings {
                auto_pick: true,
                ..fake_store_settings()
            },
            ..test_settings()
        },
    )
}

async fn chosen(app: &TestApp) -> Vec<Value> {
    let (status, body) = app.get("/api/item-selections").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body.as_array().unwrap().clone()
}

#[sqlx::test]
async fn a_new_item_gets_the_cheapest_per_unit_product_at_each_store(pool: PgPool) {
    let app = auto_pick_app(pool);

    app.add_item(&json!({ "name": "milk" })).await;

    let saved = chosen(&app).await;
    assert_eq!(saved.len(), 1, "{saved:?}");
    let ids: Vec<&str> = std::iter::once(&saved[0])
        .chain(saved[0]["also_chosen"].as_array().unwrap())
        .map(|s| s["product_id"].as_str().unwrap())
        .collect();
    // $1.55/L beats the 3L at Woolworths; Coles has one milk.
    assert!(ids.contains(&"w-milk-2l"), "{ids:?}");
    assert!(ids.contains(&"c-milk-3l"), "{ids:?}");
}

#[sqlx::test]
async fn nothing_is_picked_when_it_is_switched_off(pool: PgPool) {
    let app = TestApp::new(pool);

    app.add_item(&json!({ "name": "milk" })).await;

    assert!(chosen(&app).await.is_empty());
}
