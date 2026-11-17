//! Delivery timing through the household planner and store-tab report API.

mod common;

use axum::http::StatusCode;
use chrono::Duration;
use common::trolley::{chosen_item, store_tab_post_from, COLES_ORIGIN, WOOLWORTHS_ORIGIN};
use common::{TestApp, TEST_STORE_TAB_SECRET};
use grocery_backend::services::delivery_day_fees::store_today;
use serde_json::{json, Value};
use sqlx::PgPool;

#[sqlx::test]
async fn timing_settings_can_be_saved_edited_and_cleared(pool: PgPool) {
    let app = TestApp::new(pool);
    for (need_by, hours, status) in [
        (Some("2026-11-20T18:00:00"), Some(4), StatusCode::OK),
        (Some("2026-11-21T19:00:00"), Some(168), StatusCode::OK),
        (
            Some("2026-11-21T19:00:00"),
            Some(0),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (None, Some(4), StatusCode::UNPROCESSABLE_ENTITY),
        (
            Some("2026-10-04T02:30:00"),
            None,
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (None, None, StatusCode::OK),
    ] {
        let (actual, body) = app.put_json(
            "/api/delivery-settings",
            &json!({"stores": [], "mode": "minimise_total", "need_by": need_by, "perishable_hours": hours}),
        ).await;
        assert_eq!(actual, status, "{need_by:?} {hours:?} {body}");
        if status == StatusCode::OK {
            let (_, saved) = app.get("/api/delivery-settings").await;
            assert_eq!(saved["need_by"], json!(need_by));
            assert_eq!(saved["perishable_hours"], json!(hours));
        }
    }
}

async fn report_windows(app: &TestApp, store: &str, windows: Value) {
    let (status, body, _) = store_tab_post_from(
        app,
        if store == "woolworths" {
            WOOLWORTHS_ORIGIN
        } else {
            COLES_ORIGIN
        },
        "/api/store-tab/delivery-windows",
        &json!({"secret": TEST_STORE_TAB_SECRET, "store": store, "windows": windows}),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
}

#[sqlx::test]
async fn recommends_only_windows_inside_the_bounds_and_rejects_stale_reports(pool: PgPool) {
    let app = TestApp::new(pool.clone());
    let item = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    let (status, marked) = app
        .patch_json(
            &format!("/api/grocery-items/{item}"),
            &json!({"perishable": true}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{marked}");
    assert_eq!(marked["perishable"], true);
    app.post("/api/grocery-items/commit").await;
    let day = store_today() + Duration::days(2);
    let at = |time| format!("{day}T{time}:00");
    let window = |start, end, fee| json!({"start": at(start), "end": at(end), "fee": fee});
    let (status, _) = app.put_json(
        "/api/delivery-settings",
        &json!({"stores": [], "mode": "minimise_total", "need_by": at("16:00"), "perishable_hours": 4}),
    ).await;
    assert_eq!(status, StatusCode::OK);
    report_windows(
        &app,
        "woolworths",
        json!([
            window("11:59", "13:00", "1"),
            window("13:00", "16:01", "2"),
            window("12:00", "16:00", "7")
        ]),
    )
    .await;

    let (_, plan) = app.get("/api/order-options").await;
    let option = &plan["options"][0];
    assert_eq!(option["recommended"], true, "{plan}");
    assert_eq!(option["delivery_total"], "7");
    assert_eq!(option["stores"][0]["delivery_window"]["start"], at("12:00"));
    assert_eq!(option["stores"][0]["delivery_window"]["end"], at("16:00"));

    sqlx::query(
        "UPDATE store_delivery_window_snapshots SET reported_at = now() - interval '16 minutes'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (_, plan) = app.get("/api/order-options").await;
    assert_eq!(plan["options"][0]["recommended"], false, "{plan}");
    assert_eq!(
        plan["options"][0]["stores"][0]["delivery_problem"],
        "windows_unavailable"
    );

    report_windows(&app, "woolworths", json!([window("13:00", "16:01", "2")])).await;
    let (_, plan) = app.get("/api/order-options").await;
    assert_eq!(plan["options"][0]["recommended"], false, "{plan}");
    assert_eq!(
        plan["options"][0]["stores"][0]["delivery_problem"],
        "no_matching_window"
    );
}

#[sqlx::test]
async fn a_split_can_receive_nonperishables_earlier_than_perishables(pool: PgPool) {
    let app = TestApp::new(pool);
    let milk = chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    let bananas = chosen_item(&app, "bananas", 1, "woolworths", "w-banana").await;
    let (status, body) = app
        .patch_json(
            &format!("/api/grocery-items/{milk}"),
            &json!({"perishable": true}),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    for (id, product) in [(&milk, "c-milk-3l"), (&bananas, "c-banana")] {
        let (status, body) = app
            .put_json(
                &format!("/api/grocery-items/{id}/selection"),
                &json!({"store": "coles", "product_id": product}),
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{body}");
    }
    app.post("/api/grocery-items/commit").await;
    let day = store_today() + Duration::days(2);
    let at = |time| format!("{day}T{time}:00");
    let allowed_day = day.format("%A").to_string().to_ascii_lowercase();
    let (status, body) = app
        .put_json(
            "/api/delivery-settings",
            &json!({
                "stores": [{"store": "coles", "minimum_order": "5"}],
                "mode": "minimise_total", "need_by": at("17:00"),
                "perishable_hours": 3, "allowed_delivery_days": [allowed_day]
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let early_day = day.pred_opt().unwrap();
    report_windows(
        &app,
        "woolworths",
        json!([
            {"start": format!("{early_day}T23:00:00"), "end": at("00:30"), "fee": "0"},
            {"start": at("09:00"), "end": at("10:00"), "fee": "0"}
        ]),
    )
    .await;
    report_windows(
        &app,
        "coles",
        json!([
            {"start": at("15:00"), "end": at("16:00"), "fee": "2"}
        ]),
    )
    .await;

    let (_, plan) = app.get("/api/order-options?within=2").await;
    let woolworths = plan["options"]
        .as_array()
        .unwrap()
        .iter()
        .find(|option| option["kind"] == "woolworths")
        .unwrap();
    assert_eq!(woolworths["recommended"], false);
    assert_eq!(
        woolworths["stores"][0]["delivery_problem"],
        "no_matching_window"
    );
    assert_eq!(plan["options"][0]["kind"], "coles", "{plan}");
    assert_eq!(plan["options"][0]["recommended"], true, "{plan}");
    // The split and all-at-Coles could both make the deadline. Lower the
    // store minimum to make the split eligible for recommendation.
    let (status, body) = app
        .put_json(
            "/api/delivery-settings",
            &json!({
                "stores": [{"store": "coles", "minimum_order": null, "delivery_fee": "9"}],
                "mode": "minimise_total", "need_by": at("17:00"),
                "perishable_hours": 3, "allowed_delivery_days": [allowed_day]
            }),
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (_, earlier) = app.get("/api/order-options?within=1").await;
    assert_eq!(earlier["delivery_problem"], "no_allowed_day_in_range");
    assert!(earlier["options"]
        .as_array()
        .unwrap()
        .iter()
        .all(|option| option["recommended"] == false));

    let (_, plan) = app.get("/api/order-options?within=2").await;
    let option = &plan["options"][0];
    assert_eq!(option["kind"], "split", "{plan}");
    assert_eq!(option["recommended"], true, "{plan}");
    assert_eq!(option["stores"][0]["delivery_window"]["start"], at("09:00"));
    assert_eq!(option["stores"][1]["delivery_window"]["start"], at("15:00"));
}

#[sqlx::test]
async fn the_store_window_report_checks_its_secret_origin_and_bounds(pool: PgPool) {
    let app = TestApp::new(pool);
    let date = store_today().succ_opt().unwrap();
    let mut body = json!({"secret": "wrong", "store": "woolworths", "windows": []});
    let request = "/api/store-tab/delivery-windows";
    let (status, _, _) = store_tab_post_from(&app, WOOLWORTHS_ORIGIN, request, &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    body["secret"] = json!(TEST_STORE_TAB_SECRET);
    let (status, _, _) = store_tab_post_from(&app, COLES_ORIGIN, request, &body).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    body["windows"] = json!([{
        "start": format!("{date}T18:00:00"), "end": format!("{date}T17:00:00"), "fee": "5"
    }]);
    let (status, _, _) = store_tab_post_from(&app, WOOLWORTHS_ORIGIN, request, &body).await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}
