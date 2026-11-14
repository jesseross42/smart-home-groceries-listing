//! Bodies for the OpenAI triage key on Settings › Help.
//!
//! The key only ever travels towards the server. Answers say whether one is
//! set, never what it is.

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::services::openai_key::KeyStatus;

/// `PUT /api/triage/openai-key`.
#[derive(Deserialize, Validate)]
pub struct SaveOpenAiKeyBody {
    #[validate(length(min = 1, max = 500))]
    pub api_key: String,
}

impl fmt::Debug for SaveOpenAiKeyBody {
    /// Never prints the key.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SaveOpenAiKeyBody")
            .field("api_key", &"<redacted>")
            .finish()
    }
}

/// What the web app may know about the key.
#[derive(Debug, Serialize)]
pub struct OpenAiKeyStatusResponse {
    /// A key is saved in the database (it wins over the environment's).
    pub saved: bool,
    /// `OPENAI_API_KEY` is set, and is used while no key is saved.
    pub from_environment: bool,
    /// When the saved key last changed.
    pub changed_at: Option<DateTime<Utc>>,
}

impl OpenAiKeyStatusResponse {
    /// `status`, with whether the environment also has a key.
    pub fn new(status: KeyStatus, from_environment: bool) -> Self {
        Self {
            saved: status.saved,
            from_environment,
            changed_at: status.changed_at,
        }
    }
}
