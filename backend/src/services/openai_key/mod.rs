//! The OpenAI triage key, saved from Settings › Help.
//!
//! The key is encrypted at rest with the credential encryption key
//! (`services/encryption.rs`). It is never returned to the browser and never
//! logged: a person can only replace it, remove it, or undo the last change.
//!
//! A saved key wins over `OPENAI_API_KEY`, which stays as the fallback so a
//! server set up before this existed keeps working ([`source`]).
//!
//! SQL lives in [`repository`].

mod log;
pub mod repository;
pub mod source;

use chrono::{DateTime, Utc};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::services::encryption::Encryptor;

/// What a person may know about the key: whether there is one, never it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyStatus {
    /// A key is saved in the database.
    pub saved: bool,
    /// When the saved key last changed (saved, removed or undone).
    pub changed_at: Option<DateTime<Utc>>,
}

/// Saves, removes and reads the key.
pub struct OpenAiKeyService<'a> {
    pool: &'a PgPool,
    encryptor: Option<&'a Encryptor>,
}

impl<'a> OpenAiKeyService<'a> {
    /// Borrows what one request needs.
    pub fn new(pool: &'a PgPool, encryptor: Option<&'a Encryptor>) -> Self {
        Self { pool, encryptor }
    }

    /// Whether a key is saved.
    pub async fn status(&self) -> Result<KeyStatus, ApiError> {
        let row = repository::find(self.pool).await?;
        Ok(KeyStatus {
            saved: row.as_ref().is_some_and(|r| r.api_key_encrypted.is_some()),
            changed_at: row.map(|r| r.updated_at),
        })
    }

    /// Saves `api_key`, replacing the one before. 422 when it is empty;
    /// 500 when `CREDENTIAL_ENCRYPTION_KEY` is not set.
    pub async fn save(&self, api_key: &str, user_id: &str) -> Result<KeyStatus, ApiError> {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Err(ApiError::UnprocessableEntity(
                "The OpenAI key is empty.".into(),
            ));
        }
        let sealed = self
            .encryptor()?
            .encrypt(api_key)
            .map_err(|err| ApiError::Internal(err.into()))?;
        repository::replace(self.pool, Some(&sealed), user_id).await?;
        log::changed("saved", user_id);
        self.status().await
    }

    /// Removes the saved key. Not an error when there is none.
    pub async fn remove(&self, user_id: &str) -> Result<KeyStatus, ApiError> {
        if repository::find(self.pool).await?.is_some() {
            repository::replace(self.pool, None, user_id).await?;
        }
        log::changed("removed", user_id);
        self.status().await
    }

    /// Undoes the last save or removal (or the last undo).
    pub async fn undo(&self, user_id: &str) -> Result<KeyStatus, ApiError> {
        repository::swap_with_previous(self.pool, user_id).await?;
        log::changed("last change undone", user_id);
        self.status().await
    }

    /// The saved key, decrypted. `None` when there is none.
    pub async fn saved_key(&self) -> Result<Option<String>, ApiError> {
        let Some(sealed) = repository::find(self.pool)
            .await?
            .and_then(|row| row.api_key_encrypted)
        else {
            return Ok(None);
        };
        self.encryptor()?
            .decrypt(&sealed)
            .map(Some)
            .map_err(|err| ApiError::Internal(err.into()))
    }

    fn encryptor(&self) -> Result<&Encryptor, ApiError> {
        self.encryptor
            .ok_or_else(|| ApiError::Misconfigured("CREDENTIAL_ENCRYPTION_KEY is not set".into()))
    }
}
