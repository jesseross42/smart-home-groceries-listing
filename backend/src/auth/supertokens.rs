//! Delegates session verification to the server-side SuperTokens SDK.

use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::Deserialize;

use super::{AuthProvider, AuthUser, BoxFuture};
use crate::error::ApiError;

/// Authenticates grocery API requests through the private verification endpoint.
pub struct SuperTokensAuthProvider {
    verifier_url: String,
    secret: String,
    http: Client,
}

impl std::fmt::Debug for SuperTokensAuthProvider {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SuperTokensAuthProvider")
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
struct VerifiedSession {
    id: String,
    email: Option<String>,
}

impl SuperTokensAuthProvider {
    /// Builds a client with bounded timeouts and redirects disabled.
    pub fn new(verifier_url: String, secret: String) -> Self {
        Self {
            verifier_url,
            secret,
            http: Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(5))
                .build()
                .expect("build SuperTokens verification client"),
        }
    }

    async fn verify(&self, token: &str) -> Result<AuthUser, ApiError> {
        if self.verifier_url.is_empty() || self.secret.is_empty() {
            return Err(ApiError::Misconfigured(
                "SuperTokens verifier is not configured".into(),
            ));
        }
        let unavailable =
            || ApiError::ServiceUnavailable("Session verification unavailable".into());
        let response = self
            .http
            .post(&self.verifier_url)
            .bearer_auth(token)
            .header("x-grocery-auth-secret", &self.secret)
            .send()
            .await
            .map_err(|_| unavailable())?;
        match response.status() {
            StatusCode::UNAUTHORIZED => {
                Err(ApiError::Unauthorized("Invalid or expired session".into()))
            }
            StatusCode::OK => {
                let session: VerifiedSession = response.json().await.map_err(|_| unavailable())?;
                if session.id.trim().is_empty() {
                    return Err(unavailable());
                }
                Ok(AuthUser {
                    id: session.id,
                    email: session.email,
                })
            }
            _ => Err(unavailable()),
        }
    }
}

impl AuthProvider for SuperTokensAuthProvider {
    fn verify_token<'a>(&'a self, token: &'a str) -> BoxFuture<'a, Result<AuthUser, ApiError>> {
        Box::pin(self.verify(token))
    }
}
