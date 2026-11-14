//! Row type for `openai_triage_key`.

use chrono::{DateTime, Utc};

/// The saved OpenAI triage key, still encrypted.
#[derive(Clone, sqlx::FromRow)]
pub struct OpenAiKeyRow {
    /// `None` = the key was removed.
    pub api_key_encrypted: Option<String>,
    /// What the last change replaced, for its Undo.
    pub previous_api_key_encrypted: Option<String>,
    pub updated_by: String,
    pub updated_at: DateTime<Utc>,
}
