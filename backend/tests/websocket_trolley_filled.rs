//! The `trolley_filled` event on `/ws`: sent once when a store tab reports,
//! so the desktop app can notify even when the handoff's sheet is closed.

mod common;

use std::time::Duration;

use axum::http::StatusCode;
use common::trolley::{chosen_item, claim, create, report};
use common::TestApp;
use futures_util::StreamExt;
use serde_json::{json, Value};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::Message;

/// Long enough for a broadcast that was sent to arrive.
const QUIET_FOR: Duration = Duration::from_millis(300);

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// The `trolley_filled` events that arrive before the socket goes quiet.
async fn trolley_events(socket: &mut Socket) -> Vec<Value> {
    let mut events = Vec::new();
    while let Ok(Some(frame)) = tokio::time::timeout(QUIET_FOR, socket.next()).await {
        if let Message::Text(text) = frame.expect("a readable frame") {
            let event: Value = serde_json::from_str(&text).expect("JSON");
            if event["type"] == "trolley_filled" {
                events.push(event);
            }
        }
    }
    events
}

#[sqlx::test]
async fn a_report_announces_the_filled_trolley_once(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "toilet paper", 1, "woolworths", "w-tp-3ply").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();
    let address = app.serve().await;
    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .unwrap();
    let lines = json!({ "lines": [
        { "product_id": "w-milk-2l", "outcome": "added" },
        { "product_id": "w-tp-3ply", "outcome": "failed", "problem": "Out of stock" }
    ]});

    let (first, _, _) = report(&app, id, lines.clone()).await;
    let (second, _, _) = report(&app, id, lines).await;

    assert_eq!(first, StatusCode::OK);
    assert_eq!(second, StatusCode::CONFLICT);
    assert_eq!(
        trolley_events(&mut socket).await,
        [json!({
            "type": "trolley_filled",
            "handoff_id": id,
            "store_name": "Woolworths",
            "added": 1,
            "failed": 1
        })]
    );
}

#[sqlx::test]
async fn a_refused_report_announces_nothing(pool: PgPool) {
    let app = TestApp::new(pool);
    chosen_item(&app, "milk", 1, "woolworths", "w-milk-2l").await;
    chosen_item(&app, "toilet paper", 1, "woolworths", "w-tp-3ply").await;
    create(&app, "woolworths").await;
    let (_, claimed, _) = claim(&app).await;
    let id = claimed["handoff_id"].as_str().unwrap();
    let address = app.serve().await;
    let (mut socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/ws"))
        .await
        .unwrap();

    let only_milk = json!({ "lines": [{ "product_id": "w-milk-2l", "outcome": "added" }] });
    let (status, _, _) = report(&app, id, only_milk).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(trolley_events(&mut socket).await.is_empty());
}
