//! The key the OpenAI classifier sends: the saved one, else the
//! environment's.
//!
//! Read on every classification, so a key saved in Settings › Help is used
//! by the next item without a restart.

use sqlx::PgPool;

use super::{log, OpenAiKeyService};
use crate::services::encryption::Encryptor;
use crate::services::stores::client::BoxFuture;
use crate::services::triage::{KeySource, TriageError};

/// The saved key first, then `OPENAI_API_KEY`.
pub struct SavedOrEnvironmentKey {
    pool: PgPool,
    encryptor: Option<Encryptor>,
    environment_key: String,
}

impl SavedOrEnvironmentKey {
    /// `environment_key` is `OPENAI_API_KEY`, empty when not set.
    pub fn new(pool: PgPool, encryptor: Option<Encryptor>, environment_key: &str) -> Self {
        Self {
            pool,
            encryptor,
            environment_key: environment_key.to_string(),
        }
    }
}

impl KeySource for SavedOrEnvironmentKey {
    fn key(&self) -> BoxFuture<'_, Result<Option<String>, TriageError>> {
        Box::pin(async move {
            let saved = OpenAiKeyService::new(&self.pool, self.encryptor.as_ref())
                .saved_key()
                .await
                .unwrap_or_else(|err| {
                    log::unreadable(&err);
                    None
                });
            match saved {
                Some(key) => Ok(Some(key)),
                None if !self.environment_key.is_empty() => Ok(Some(self.environment_key.clone())),
                None => Err(TriageError::Unavailable(
                    "no OpenAI key is saved in Settings › Help".to_string(),
                )),
            }
        })
    }
}
