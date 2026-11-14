//! The `grocery_list_changed` event on `/ws`: open pages are told to read
//! the list again after a write that may have changed it.

mod common;

use std::time::Duration;

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use common::trolley::{chosen_item, claim, create, report};
use common::{TestApp, TEST_BRIDGE_SECRET};
use futures_util::StreamExt;
use grocery_backend::routes::alexa::BRIDGE_SECRET_HEADER;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::Message;

const LIST_CHANGED: &str = "grocery_list_changed";
/// Long enough for a broadcast that was sent to arrive.
const QUIET_FOR: Duration = Duration::from_millis(300);

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// Opens a session and reads its opening counts, so later frames are events.
async fn connect(app: &TestApp) -> Socket {
    let address = app.serve().await;
    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .unwrap();
    let snapshot = next_frame(&mut socket, Duration::from_secs(5))
        .await
        .expect("the opening counts");
    assert_eq!(snapshot["type"], "counts_snapshot");
    socket
}

/// The next text frame as JSON, or `None` when nothing arrives in `wait`.
async fn next_frame(socket: &mut Socket, wait: Duration) -> Option<Value> {
    let frame = tokio::time::timeout(wait, socket.next()).await.ok()??;
    match frame.expect("the frame should be readable") {
        Message::Text(text) => Some(serde_json::from_str(&text).expect("JSON")),
        other => panic!("expected a text frame, got {other:?}"),
    }
}

/// Whether a list-changed event arrives soon, passing over count events.
async fn list_changed_arrives(socket: &mut Socket) -> bool {
    while let Some(event) = next_frame(socket, QUIET_FOR).await {
        if event["type"] == LIST_CHANGED {
            return true;
        }
    }
    false
}

#[sqlx::test]
async fn adding_editing_and_deleting_an_item_each_tell_open_pages(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut socket = connect(&app).await;

    let id = app
        .add_item(&json!({ "name": "milk", "quantity": 1 }))
        .await;
    assert!(list_changed_arrives(&mut socket).await, "add");

    app.patch_json(
        &format!("/api/grocery-items/{id}"),
        &json!({ "quantity": 2 }),
    )
    .await;
    assert!(list_changed_arrives(&mut socket).await, "edit");

    app.delete(&format!("/api/grocery-items/{id}")).await;
    assert!(list_changed_arrives(&mut socket).await, "delete");
}

#[sqlx::test]
async fn commit_and_release_tell_open_pages(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "quantity": 1 }))
        .await;
    let mut socket = connect(&app).await;

    app.post("/api/grocery-items/commit").await;
    assert!(list_changed_arrives(&mut socket).await, "commit");

    app.post("/api/grocery-items/release").await;
    assert!(list_changed_arrives(&mut socket).await, "release");
}

#[sqlx::test]
async fn accepting_a_pending_request_tells_open_pages(pool: PgPool) {
    let app = TestApp::new(pool);
    let pending = app.create_pending("bread", 1).await;
    let mut socket = connect(&app).await;

    app.post(&format!("/api/voice-requests/{pending}/accept"))
        .await;

    assert!(list_changed_arrives(&mut socket).await);
}

#[sqlx::test]
async fn a_voice_removal_tells_open_pages(pool: PgPool) {
    let app = TestApp::new(pool);
    app.add_item(&json!({ "name": "milk", "quantity": 1 }))
        .await;
    let mut socket = connect(&app).await;

    app.send(
        Request::builder()
            .method(Method::POST)
            .uri("/api/intake/alexa/remove")
            .header("content-type", "application/json")
            .header(BRIDGE_SECRET_HEADER, TEST_BRIDGE_SECRET)
            .body(Body::from(json!({ "item": "milk" }).to_string()))
            .unwrap(),
    )
    .await;

    assert!(list_changed_arrives(&mut socket).await);
}

#[sqlx::test]
async fn reads_and_refused_writes_stay_quiet(pool: PgPool) {
    let app = TestApp::new(pool);
    let mut socket = connect(&app).await;

    app.get("/api/grocery-items").await;
    app.delete("/api/grocery-items/00000000-0000-0000-0000-000000000001")
        .await;
    app.add_item(&json!({ "name": "milk", "quantity": 1 }))
        .await;

    // Exactly one event: the add. The read and the 404 sent none before it.
    assert!(list_changed_arrives(&mut socket).await);
    assert!(!list_changed_arrives(&mut socket).await);
}

#[sqlx::test]
async fn read_only_posts_stay_quiet(pool: PgPool) {
    let app = TestApp::new(pool);
    let id = app
        .add_item(&json!({ "name": "milk", "quantity": 1 }))
        .await;
    let mut socket = connect(&app).await;

    let (history, _) = app
        .post_json(
            "/api/purchase-history/products",
            &json!({ "products": [{ "store": "coles", "product_id": "c-milk-3l" }] }),
        )
        .await;
    let (snapshot, _) = app
        .post(&format!("/api/grocery-items/{id}/selection/snapshot"))
        .await;

    assert_eq!(history, StatusCode::OK);
    assert!(snapshot.is_success(), "{snapshot}");
    assert!(!list_changed_arrives(&mut socket).await);
}

#[sqlx::test]
async fn a_filled_trolley_tells_open_pages_once_its_items_are_recorded(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();
    let mut socket = connect(&app).await;

    let lines = json!({ "lines": [{ "product_id": "w-milk-2l", "outcome": "added" }] });
    let (status, _, _) = report(&app, id, lines).await;

    assert_eq!(status, StatusCode::OK);
    // Recording runs after the report is answered; wait for its event.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    let mut told = false;
    while !told && tokio::time::Instant::now() < deadline {
        told = list_changed_arrives(&mut socket).await;
    }
    assert!(told, "recording the purchase should tell open pages");
}
