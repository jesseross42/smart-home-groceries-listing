//! Extracts the credential offered separately from the WebSocket event protocol.

use axum::extract::FromRequestParts;
use axum::http::{header::SEC_WEBSOCKET_PROTOCOL, request::Parts, HeaderMap};

use super::{authenticate, bearer_token, record_for_access_log, AuthUser};
use crate::error::ApiError;
use crate::state::AppState;

/// The only subprotocol the server can negotiate.
pub const EVENT_PROTOCOL: &str = "grocery.events";

fn offered_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get_all(SEC_WEBSOCKET_PROTOCOL)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .find_map(|value| {
            value
                .trim()
                .strip_prefix("grocery.session.")
                .filter(|token| !token.is_empty())
        })
}

/// The authenticated WebSocket credential, retained for periodic revalidation.
pub struct SocketUser {
    token: Option<String>,
}

impl SocketUser {
    /// Checks whether this connection still has an active session.
    pub async fn is_active(&self, state: &AppState) -> bool {
        authenticate(state, self.token.as_deref()).await.is_ok()
    }
}

impl FromRequestParts<AppState> for SocketUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = bearer_token(parts).or_else(|| offered_token(&parts.headers));
        let user: AuthUser = authenticate(state, token).await?;
        record_for_access_log(parts, &user);
        Ok(Self {
            token: token.map(str::to_owned),
        })
    }
}
