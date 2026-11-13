//! Exercises the private verifier contract through Axum's REST and WebSocket routes.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use common::session::{serve_verifier, TOKEN};
use common::TestApp;
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::{client::IntoClientRequest, Error};

fn grocery_request(token: &str) -> Request<Body> {
    Request::get("/api/grocery-items")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap()
}

#[sqlx::test]
async fn valid_sessions_can_read_the_list(pool: PgPool) {
    let server = serve_verifier().await;
    let app = TestApp::with_verifier(pool, &server);
    let (status, body) = app.send(grocery_request(TOKEN)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[sqlx::test]
async fn verifier_failure_never_grants_access(pool: PgPool) {
    let server = serve_verifier().await;
    let app = TestApp::with_verifier(pool, &server);
    server.reset().await;
    let (status, _) = app.send(grocery_request(TOKEN)).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[sqlx::test]
async fn websocket_negotiates_only_the_event_protocol(pool: PgPool) {
    let server = serve_verifier().await;
    let app = TestApp::with_verifier(pool, &server);
    let address = app.serve().await;
    let mut request = format!("ws://{address}/ws").into_client_request().unwrap();
    request.headers_mut().insert(
        "sec-websocket-protocol",
        format!("grocery.events, grocery.session.{TOKEN}")
            .parse()
            .unwrap(),
    );
    let (_, response) = tokio_tungstenite::connect_async(request).await.unwrap();
    assert_eq!(
        response.headers()["sec-websocket-protocol"],
        "grocery.events"
    );
}

#[sqlx::test]
async fn websocket_ignores_credentials_in_the_url(pool: PgPool) {
    let server = serve_verifier().await;
    let app = TestApp::with_verifier(pool, &server);
    let address = app.serve().await;
    let result = tokio_tungstenite::connect_async(format!("ws://{address}/ws?token={TOKEN}")).await;
    assert!(
        matches!(result, Err(Error::Http(response)) if response.status() == StatusCode::UNAUTHORIZED)
    );
}
