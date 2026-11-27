//! The `trolley_filled` event on `/ws`: when the store tab reports a fill,
//! every open session hears it, so the desktop app can notify even when the
//! Send sheet is closed. A refused second report sends nothing.

mod common;

use std::time::Duration;

use common::trolley::{chosen_item, claim, create, store_tab_post};
use common::{TestApp, TEST_STORE_TAB_SECRET};
use futures_util::StreamExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::Message;

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
    let snapshot = next_frame(&mut socket, Duration::from_secs(5)).await;
    assert_eq!(
        snapshot.expect("the opening counts")["type"],
        "counts_snapshot"
    );
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

/// Every `trolley_filled` event that arrives soon, passing over the rest.
async fn trolley_filled_events(socket: &mut Socket) -> Vec<Value> {
    let mut found = Vec::new();
    while let Some(event) = next_frame(socket, QUIET_FOR).await {
        if event["type"] == "trolley_filled" {
            found.push(event);
        }
    }
    found
}

/// A claimed Woolworths handoff for milk and toilet paper, and its report URI.
async fn claimed_handoff(app: &TestApp) -> (String, String) {
    chosen_item(app, "milk", 1, "woolworths", "w-milk-2l").await;
    chosen_item(app, "toilet paper", 1, "woolworths", "w-tp-3ply").await;
    create(app, "woolworths").await;
    let (_, claimed, _) = claim(app).await;
    let id = claimed["handoff_id"].as_str().unwrap().to_string();
    let uri = format!("/api/store-tab/trolley-handoffs/{id}/report");
    (id, uri)
}

fn one_failed() -> Value {
    json!({ "secret": TEST_STORE_TAB_SECRET, "lines": [
        { "product_id": "w-milk-2l", "outcome": "added" },
        { "product_id": "w-tp-3ply", "outcome": "failed", "problem": "Out of stock" }
    ]})
}

#[sqlx::test]
async fn a_reported_fill_is_pushed_with_its_counts(pool: PgPool) {
    let app = TestApp::new(pool);
    let (id, uri) = claimed_handoff(&app).await;
    let mut socket = connect(&app).await;

    let (status, body, _) = store_tab_post(&app, &uri, &one_failed()).await;
    assert!(status.is_success(), "{body}");

    let events = trolley_filled_events(&mut socket).await;
    assert_eq!(
        events,
        vec![json!({
            "type": "trolley_filled", "handoff_id": id, "store": "woolworths",
            "store_name": "Woolworths", "added": 1, "not_added": 1,
        })]
    );
}

#[sqlx::test]
async fn a_refused_second_report_pushes_nothing(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, uri) = claimed_handoff(&app).await;
    store_tab_post(&app, &uri, &one_failed()).await;
    let mut socket = connect(&app).await;

    let (status, _, _) = store_tab_post(&app, &uri, &one_failed()).await;

    assert!(status.is_client_error());
    assert!(trolley_filled_events(&mut socket).await.is_empty());
}

#[sqlx::test]
async fn a_session_opened_after_the_fill_is_not_told_about_it(pool: PgPool) {
    let app = TestApp::new(pool);
    let (_, uri) = claimed_handoff(&app).await;
    store_tab_post(&app, &uri, &one_failed()).await;

    let mut socket = connect(&app).await;

    assert!(trolley_filled_events(&mut socket).await.is_empty());
}
