//! Delivery fees for a day (issue #70): the bookmark reports the cheapest fee
//! each store showed per day, and `GET /api/order-options?day=` plans with
//! the fee on that day instead of the typed one.
//!
//! Fake catalogue: milk $3.10 + bananas $0.80 at Woolworths ($3.90), $4.95 +
//! $0.83 at Coles ($5.78). Typed fees are $9 at both.

mod common;

use axum::http::StatusCode;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use common::trolley::{chosen_item, claim, create, store_tab_post};
use common::{TestApp, TEST_STORE_TAB_SECRET};
use grocery_backend::services::delivery_day_fees::store_today;
use serde_json::{json, Value};
use sqlx::PgPool;

/// The first `weekday` after today, in the stores' time zone.
fn next(weekday: Weekday) -> NaiveDate {
    let mut day = store_today() + Duration::days(1);
    while day.weekday() != weekday {
        day += Duration::days(1);
    }
    day
}

async fn report_days(pool: &PgPool, store: &str, days: &[(NaiveDate, &str)]) {
    for (date, fee) in days {
        sqlx::query(
            "INSERT INTO store_delivery_day_fees (store, delivery_date, cheapest_fee)
             VALUES ($1, $2, $3::numeric)",
        )
        .bind(store)
        .bind(date)
        .bind(fee)
        .execute(pool)
        .await
        .unwrap();
    }
}

/// Milk and bananas chosen at both stores, committed, $9 typed fees.
async fn order_at_both_stores(app: &TestApp) {
    for (name, woolworths, coles) in [
        ("milk", "w-milk-2l", "c-milk-3l"),
        ("bananas", "w-banana", "c-banana"),
    ] {
        let id = chosen_item(app, name, 1, "woolworths", woolworths).await;
        let (status, body) = app
            .put_json(
                &format!("/api/grocery-items/{id}/selection"),
                &json!({ "store": "coles", "product_id": coles }),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    app.post("/api/grocery-items/commit").await;
    let rules = |store| json!({ "store": store, "delivery_fee": "9" });
    let (status, body) = app
        .put_json(
            "/api/delivery-settings",
            &json!({ "stores": [rules("woolworths"), rules("coles")], "mode": "minimise_total" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn options(app: &TestApp, query: &str) -> Value {
    let (status, body) = app.get(&format!("/api/order-options{query}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test]
async fn the_fee_on_the_chosen_day_decides_the_store(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    order_at_both_stores(&app).await;
    let tuesday = next(Weekday::Tue);
    let saturday = next(Weekday::Sat);
    report_days(&pool, "woolworths", &[(tuesday, "15"), (saturday, "0")]).await;

    let on_tuesday = options(&app, "?day=tuesday").await;
    assert_eq!(on_tuesday["delivery_day"], "tuesday");
    assert_eq!(on_tuesday["options"][0]["kind"], "coles", "{on_tuesday}");
    assert_eq!(on_tuesday["store_fees"][0]["date"], tuesday.to_string());
    assert_eq!(on_tuesday["store_fees"][0]["fee"], "15");

    let on_saturday = options(&app, "?day=saturday").await;
    assert_eq!(
        on_saturday["options"][0]["kind"], "woolworths",
        "{on_saturday}"
    );
    assert_eq!(on_saturday["options"][0]["delivery_total"], "0");

    // No day: each store's cheapest day.
    let any_day = options(&app, "").await;
    assert_eq!(any_day["delivery_day"], Value::Null);
    assert_eq!(any_day["options"][0]["kind"], "woolworths");
}

#[sqlx::test]
async fn a_day_the_store_did_not_report_keeps_the_typed_fee(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    order_at_both_stores(&app).await;
    report_days(&pool, "woolworths", &[(next(Weekday::Tue), "15")]).await;

    let on_friday = options(&app, "?day=friday").await;

    assert_eq!(on_friday["store_fees"], json!([]));
    assert_eq!(on_friday["options"][0]["kind"], "woolworths");
    assert_eq!(on_friday["options"][0]["delivery_total"], "9");
}

#[sqlx::test]
async fn the_usual_day_is_used_unless_another_or_any_day_is_asked(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    order_at_both_stores(&app).await;
    let rules = |store| json!({ "store": store, "delivery_fee": "9" });
    app.put_json(
        "/api/delivery-settings",
        &json!({
            "stores": [rules("woolworths"), rules("coles")],
            "mode": "minimise_total",
            "default_delivery_day": "tuesday"
        }),
    )
    .await;
    report_days(
        &pool,
        "woolworths",
        &[(next(Weekday::Tue), "15"), (next(Weekday::Sat), "0")],
    )
    .await;

    let usual = options(&app, "").await;
    assert_eq!(usual["delivery_day"], "tuesday");
    assert_eq!(usual["options"][0]["kind"], "coles", "{usual}");

    let any = options(&app, "?day=any").await;
    assert_eq!(any["delivery_day"], Value::Null);
    assert_eq!(any["options"][0]["kind"], "woolworths", "{any}");
}

#[sqlx::test]
async fn within_takes_the_cheapest_day_up_to_that_many_days_ahead(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    order_at_both_stores(&app).await;
    let today = store_today();
    let six_days = today + Duration::days(6);
    report_days(&pool, "woolworths", &[(today, "15"), (six_days, "0")]).await;

    let same_day = options(&app, "?within=0").await;
    assert_eq!(same_day["within_days"], 0);
    assert_eq!(same_day["options"][0]["kind"], "coles", "{same_day}");

    let a_week = options(&app, "?within=7").await;
    assert_eq!(a_week["options"][0]["kind"], "woolworths", "{a_week}");
    assert_eq!(a_week["store_fees"][0]["date"], six_days.to_string());
}

#[sqlx::test]
async fn an_unknown_day_or_too_many_days_is_refused(pool: PgPool) {
    let app = TestApp::new(pool);
    let (status, _) = app.get("/api/order-options?day=someday").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = app.get("/api/order-options?within=8").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn the_bookmark_report_saves_each_days_cheapest_fee(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();
    let tuesday = next(Weekday::Tue);

    let (status, body, _) = store_tab_post(
        &app,
        &format!("/api/store-tab/trolley-handoffs/{id}/report"),
        &json!({
            "secret": TEST_STORE_TAB_SECRET,
            "lines": [{ "product_id": "w-milk-2l", "outcome": "added" }],
            "delivery_days": [{ "date": tuesday, "cheapest_fee": "2" }]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "{body}");
    let (store, fee): (String, String) = sqlx::query_as(
        "SELECT store, cheapest_fee::text FROM store_delivery_day_fees WHERE delivery_date = $1",
    )
    .bind(tuesday)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((store.as_str(), fee.as_str()), ("woolworths", "2"));
}
