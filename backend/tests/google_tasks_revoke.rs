//! Disconnect takes the sign-in back at Google (issue #86), with `wiremock`
//! standing in for Google's revoke endpoint. Never the real Google.

mod common;

use axum::http::StatusCode;
use common::{google_tasks_settings, triage_settings, TestApp};
use grocery_backend::config::{GoogleTasksMode, TriageProvider};
use serde_json::json;
use sqlx::PgPool;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A connected app whose stand-in Google answers the revoke with `status`.
async fn connected(pool: PgPool, server: &MockServer, status: u16) -> TestApp {
    Mock::given(method("POST"))
        .and(path("/token"))
        .and(body_string_contains("grant_type=authorization_code"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "access-1", "expires_in": 3599, "refresh_token": "refresh-1"
        })))
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(path("/revoke"))
        .and(body_string_contains("token=refresh-1"))
        .respond_with(ResponseTemplate::new(status))
        .expect(1)
        .mount(server)
        .await;
    let app = TestApp::with_google_tasks(
        pool,
        google_tasks_settings(GoogleTasksMode::Live, &server.uri()),
        triage_settings(TriageProvider::Off, ""),
    );
    app.connect_google_tasks("good-code").await;
    app
}

async fn disconnect(app: &TestApp) -> serde_json::Value {
    let (status, body) = app.delete("/api/intake/google-tasks/sign-in").await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (_, settings) = app.get("/api/intake/settings").await;
    assert_eq!(settings["google_tasks"]["connected"], false);
    body
}

#[sqlx::test]
async fn disconnect_sends_the_refresh_token_to_googles_revoke(pool: PgPool) {
    let server = MockServer::start().await;
    let app = connected(pool, &server, 200).await;

    assert_eq!(disconnect(&app).await["revoked_at_google"], true);
}

#[sqlx::test]
async fn a_sign_in_google_already_dropped_counts_as_revoked(pool: PgPool) {
    let server = MockServer::start().await;
    let app = connected(pool, &server, 400).await;

    assert_eq!(disconnect(&app).await["revoked_at_google"], true);
}

#[sqlx::test]
async fn google_down_still_forgets_the_sign_in_and_says_so(pool: PgPool) {
    let server = MockServer::start().await;
    let app = connected(pool, &server, 503).await;

    assert_eq!(disconnect(&app).await["revoked_at_google"], false);
}

#[sqlx::test]
async fn disconnecting_twice_has_nothing_to_revoke(pool: PgPool) {
    let server = MockServer::start().await;
    let app = connected(pool, &server, 200).await;
    disconnect(&app).await;

    // The revoke mock expects one call; a second Disconnect sends none.
    assert_eq!(disconnect(&app).await["revoked_at_google"], true);
}
