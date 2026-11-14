//! Where a classifier's API key comes from.
//!
//! A local model server's key is fixed when the process starts
//! ([`FixedKey`]). The OpenAI key can be saved and replaced while the
//! backend runs (`services/openai_key`), so the client asks for it on every
//! call instead of holding a copy.

use super::model::TriageError;
use crate::services::stores::client::BoxFuture;

/// Gives the key to send, `None` to send no `Authorization` header.
pub trait KeySource: Send + Sync {
    /// The key for one request. An error holds the item for review with a
    /// reason a person can act on.
    fn key(&self) -> BoxFuture<'_, Result<Option<String>, TriageError>>;
}

/// A key set once in the environment. Empty means none.
pub struct FixedKey(String);

impl FixedKey {
    /// Holds `key` for the life of the process.
    pub fn new(key: &str) -> Self {
        Self(key.to_string())
    }
}

impl KeySource for FixedKey {
    fn key(&self) -> BoxFuture<'_, Result<Option<String>, TriageError>> {
        let key = (!self.0.is_empty()).then(|| self.0.clone());
        Box::pin(async move { Ok(key) })
    }
}
