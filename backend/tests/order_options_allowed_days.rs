//! Allowed delivery days (issue #74): `GET /api/order-options` costs delivery
//! only on the weekdays saved in Settings › Delivery, says when the delivery
//! asked for cannot come on one, and then recommends nothing.
//!
//! Fake catalogue: milk $3.10 at Woolworths, $4.95 at Coles. Typed fees are
//! $9 at both.

mod common;

use axum::http::StatusCode;
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use common::trolley::chosen_item;
use common::TestApp;
use grocery_backend::services::delivery_day_fees::store_today;
use serde_json::{json, Value};
use sqlx::PgPool;

/// The weekday name the API uses for `date`.
fn day_name(date: NaiveDate) -> &'static str {
    match date.weekday() {
        Weekday::Mon => "monday",
        Weekday::Tue => "tuesday",
        Weekday::Wed => "wednesday",
        Weekday::Thu => "thursday",
        Weekday::Fri => "friday",
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
    }
}

async fn report_day(pool: &PgPool, store: &str, date: NaiveDate, fee: &str) {
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

/// Milk chosen at both stores and committed, $9 typed fees, and delivery
/// allowed only on `allowed`.
async fn order_with_allowed_days(app: &TestApp, allowed: &[&str]) {
    let id = chosen_item(app, "milk", 1, "woolworths", "w-milk-2l").await;
    let (status, body) = app
        .put_json(
            &format!("/api/grocery-items/{id}/selection"),
            &json!({ "store": "coles", "product_id": "c-milk-3l" }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    app.post("/api/grocery-items/commit").await;
    let rules = |store| json!({ "store": store, "delivery_fee": "9" });
    let (status, body) = app
        .put_json(
            "/api/delivery-settings",
            &json!({
                "stores": [rules("woolworths"), rules("coles")],
                "mode": "minimise_total",
                "allowed_delivery_days": allowed,
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

async fn options(app: &TestApp, query: &str) -> Value {
    let (status, body) = app.get(&format!("/api/order-options{query}")).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

/// True when any option is marked recommended.
fn recommends_something(plan: &Value) -> bool {
    plan["options"]
        .as_array()
        .unwrap()
        .iter()
        .any(|option| option["recommended"] == true)
}

#[sqlx::test]
async fn a_cheaper_fee_on_a_day_that_is_not_allowed_is_ignored(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let today = store_today();
    let (allowed_day, other_day) = (today + Duration::days(2), today + Duration::days(3));
    order_with_allowed_days(&app, &[day_name(allowed_day)]).await;
    report_day(&pool, "woolworths", allowed_day, "5").await;
    report_day(&pool, "woolworths", other_day, "0").await;

    let plan = options(&app, "?day=any").await;

    assert_eq!(
        plan["allowed_delivery_days"],
        json!([day_name(allowed_day)])
    );
    assert_eq!(plan["delivery_problem"], Value::Null);
    assert_eq!(plan["store_fees"][0]["date"], allowed_day.to_string());
    assert_eq!(plan["store_fees"][0]["fee"], "5");
    assert!(recommends_something(&plan), "{plan}");
}

#[sqlx::test]
async fn a_weekday_that_is_not_allowed_is_reported_and_nothing_is_recommended(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    order_with_allowed_days(&app, &["tuesday"]).await;

    let friday = options(&app, "?day=friday").await;
    assert_eq!(friday["delivery_problem"], "day_not_allowed");
    assert!(!recommends_something(&friday), "{friday}");

    let tuesday = options(&app, "?day=tuesday").await;
    assert_eq!(tuesday["delivery_problem"], Value::Null);
    assert!(recommends_something(&tuesday), "{tuesday}");
}

#[sqlx::test]
async fn a_range_that_ends_before_an_allowed_day_is_reported(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let in_three_days = store_today() + Duration::days(3);
    order_with_allowed_days(&app, &[day_name(in_three_days)]).await;

    // One day short of the allowed day.
    let too_soon = options(&app, "?within=2").await;
    assert_eq!(too_soon["delivery_problem"], "no_allowed_day_in_range");
    assert!(!recommends_something(&too_soon), "{too_soon}");

    // The last day of the range is the allowed day.
    let in_time = options(&app, "?within=3").await;
    assert_eq!(in_time["delivery_problem"], Value::Null);
    assert!(recommends_something(&in_time), "{in_time}");
}

#[sqlx::test]
async fn no_allowed_days_saved_means_any_day(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    order_with_allowed_days(&app, &[]).await;

    let plan = options(&app, "?within=0").await;

    assert_eq!(plan["allowed_delivery_days"], json!([]));
    assert_eq!(plan["delivery_problem"], Value::Null);
    assert!(recommends_something(&plan), "{plan}");
}
