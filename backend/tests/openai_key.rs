//! The OpenAI triage key through the real router: saved encrypted from
//! Settings › Help, used by the classifier, never returned.
//!
//! `wiremock` stands in for OpenAI. The environment's key is `test-key`.

mod common;

use axum::http::StatusCode;
use common::settings::{google_tasks_settings, triage_settings};
use common::triage::Shown;
use common::TestApp;
use grocery_backend::config::{GoogleTasksMode, TriageProvider, TriageSettings};
use serde_json::{json, Value};
use sqlx::PgPool;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const SAVED_KEY: &str = "sk-saved-in-settings";
const KEY_PATH: &str = "/api/triage/openai-key";

/// A provider that approves an item only when it is sent `key`.
async fn provider_expecting(key: &str) -> MockServer {
    let server = MockServer::start().await;
    let content = r#"{"supermarket_item": true, "confidence": 0.97, "reason": "A grocery."}"#;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", format!("Bearer {key}").as_str()))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "choices": [{ "index": 0, "message": { "role": "assistant", "content": content } }]
        })))
        .mount(&server)
        .await;
    server
}

fn openai(server: &MockServer) -> TriageSettings {
    triage_settings(TriageProvider::OpenAi, &format!("{}/v1", server.uri()))
}

/// The application with an encryption key, checking items with `triage`.
fn app(pool: PgPool, triage: TriageSettings) -> TestApp {
    let google = google_tasks_settings(GoogleTasksMode::Fake, "http://127.0.0.1:9");
    TestApp::with_google_tasks(pool, google, triage)
}

async fn save(app: &TestApp, key: &str) -> Value {
    let (status, body) = app.put_json(KEY_PATH, &json!({ "api_key": key })).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body
}

#[sqlx::test]
async fn a_saved_key_is_encrypted_and_never_returned(pool: PgPool) {
    let server = provider_expecting(SAVED_KEY).await;
    let app = app(pool.clone(), openai(&server));

    let (_, before) = app.get(KEY_PATH).await;
    assert_eq!(before["saved"], false);
    assert_eq!(before["from_environment"], true);

    let saved = save(&app, SAVED_KEY).await;

    assert_eq!(saved["saved"], true);
    assert!(!saved.to_string().contains(SAVED_KEY));
    let (_, after) = app.get(KEY_PATH).await;
    assert!(!after.to_string().contains(SAVED_KEY));
    let stored: String = sqlx::query_scalar("SELECT api_key_encrypted FROM openai_triage_key")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!stored.contains(SAVED_KEY));
}

#[sqlx::test]
async fn the_classifier_uses_the_saved_key_without_a_restart(pool: PgPool) {
    let server = provider_expecting(SAVED_KEY).await;
    let app = app(pool, openai(&server));
    save(&app, SAVED_KEY).await;

    let (_, shown, _) = app.webhook_item_triaged("milk").await;

    assert_eq!(shown, Shown::Pending);
}

#[sqlx::test]
async fn removing_the_key_falls_back_to_the_environment_and_undo_restores_it(pool: PgPool) {
    let saved_only = provider_expecting(SAVED_KEY).await;
    let app = app(pool, openai(&saved_only));
    save(&app, SAVED_KEY).await;

    let (status, removed) = app.delete(KEY_PATH).await;
    assert_eq!(status, StatusCode::OK, "{removed}");
    assert_eq!(removed["saved"], false);
    // The provider only takes the saved key, so the environment's is refused.
    let (_, shown, request) = app.webhook_item_triaged("milk").await;
    assert_eq!(shown, Shown::Held, "{request}");

    let (status, undone) = app.post(&format!("{KEY_PATH}/undo")).await;
    assert_eq!(status, StatusCode::OK, "{undone}");
    assert_eq!(undone["saved"], true);
    let (_, shown, _) = app.webhook_item_triaged("bread").await;
    assert_eq!(shown, Shown::Pending);
}

#[sqlx::test]
async fn replacing_the_key_can_be_undone(pool: PgPool) {
    let server = provider_expecting(SAVED_KEY).await;
    let app = app(pool, openai(&server));
    save(&app, SAVED_KEY).await;
    save(&app, "sk-a-wrong-one").await;

    app.post(&format!("{KEY_PATH}/undo")).await;

    let (_, shown, _) = app.webhook_item_triaged("milk").await;
    assert_eq!(shown, Shown::Pending);
}

#[sqlx::test]
async fn with_no_key_anywhere_items_are_held_and_say_why(pool: PgPool) {
    let server = provider_expecting(SAVED_KEY).await;
    let triage = TriageSettings {
        api_key: String::new(),
        ..openai(&server)
    };
    let app = app(pool, triage);

    let (_, status) = app.get(KEY_PATH).await;
    assert_eq!(status["from_environment"], false);
    let (_, shown, request) = app.webhook_item_triaged("milk").await;

    assert_eq!(shown, Shown::Held);
    let reason = request["triage_reason"].as_str().unwrap();
    assert!(reason.contains("Settings › Help"), "{reason}");
}

#[sqlx::test]
async fn an_empty_key_is_refused_and_a_missing_encryption_key_is_reported(pool: PgPool) {
    let server = provider_expecting(SAVED_KEY).await;
    let with_encryption = app(pool.clone(), openai(&server));
    let (status, _) = with_encryption
        .put_json(KEY_PATH, &json!({ "api_key": "   " }))
        .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

    let without_encryption = TestApp::with_triage(pool, openai(&server));
    let (status, _) = without_encryption
        .put_json(KEY_PATH, &json!({ "api_key": SAVED_KEY }))
        .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}
