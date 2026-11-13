//! Mock the private verification boundary for tests of Axum's protected routes.

use crate::common::{test_settings, TestApp};
use grocery_backend::{build_app, config::Settings};
use sqlx::PgPool;
use wiremock::{
    matchers::{header, method, path},
    Mock, MockServer, ResponseTemplate,
};

/// A test credential accepted by the mock verifier.
pub const TOKEN: &str = "fixture.session.token";
const SECRET: &str = "test-verifier-secret";

/// Starts a verifier that accepts only the expected service and user credentials.
pub async fn serve_verifier() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/verify"))
        .and(header("x-grocery-auth-secret", SECRET))
        .respond_with(ResponseTemplate::new(401))
        .with_priority(2)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/verify"))
        .and(header("x-grocery-auth-secret", SECRET))
        .and(header("authorization", format!("Bearer {TOKEN}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({"id": "member"})))
        .with_priority(1)
        .mount(&server)
        .await;
    server
}

impl TestApp {
    /// Enables authentication without configuring a verifier.
    pub fn with_auth(pool: PgPool) -> Self {
        Self {
            router: build_app(
                pool,
                Settings {
                    dev_auth_bypass: false,
                    ..test_settings()
                },
            ),
        }
    }

    /// Builds Axum with authentication enabled and points it at a test verifier.
    pub fn with_verifier(pool: PgPool, server: &MockServer) -> Self {
        Self {
            router: build_app(
                pool,
                Settings {
                    dev_auth_bypass: false,
                    auth_verifier_url: format!("{}/verify", server.uri()),
                    auth_verifier_secret: SECRET.into(),
                    ..test_settings()
                },
            ),
        }
    }
}
