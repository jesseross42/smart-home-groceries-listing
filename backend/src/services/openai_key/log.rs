//! Log lines for the OpenAI triage key. The key itself is never logged.

/// A person saved, removed or undid a change to the key.
pub fn changed(what: &'static str, user_id: &str) {
    tracing::info!(change = what, user_id, "openai triage key changed");
}

/// The saved key could not be read, so the environment's key is used.
pub fn unreadable(err: &crate::error::ApiError) {
    tracing::warn!(%err, "saved openai triage key could not be read");
}
