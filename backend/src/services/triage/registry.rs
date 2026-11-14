//! Choosing the classifier for this process. The one place that names a
//! concrete [`TriageModel`].

use std::sync::Arc;

use sqlx::PgPool;

use super::assessor::Triage;
use super::fake::FakeTriage;
use super::key::{FixedKey, KeySource};
use super::model::TriageModel;
use super::openai::OpenAiCompatible;
use crate::config::{TriageProvider, TriageSettings};
use crate::services::encryption::Encryptor;
use crate::services::openai_key::source::SavedOrEnvironmentKey;

/// Builds the classifier `settings` asks for.
///
/// OpenAI's key is read from the database on every call (saved in Settings ›
/// Help), falling back to `OPENAI_API_KEY`; with neither, every item is held
/// with a reason that says what to fix. A local server's key is fixed.
pub fn build(settings: &TriageSettings, pool: &PgPool, encryptor: Option<&Encryptor>) -> Triage {
    let key: Arc<dyn KeySource> = match settings.provider {
        TriageProvider::OpenAi => Arc::new(SavedOrEnvironmentKey::new(
            pool.clone(),
            encryptor.cloned(),
            &settings.api_key,
        )),
        _ => Arc::new(FixedKey::new(&settings.api_key)),
    };
    let model: Option<Arc<dyn TriageModel>> = match settings.provider {
        TriageProvider::Off => None,
        TriageProvider::Fake => Some(Arc::new(FakeTriage)),
        TriageProvider::OpenAi
        | TriageProvider::Ollama
        | TriageProvider::LlamaCpp
        | TriageProvider::Vllm => Some(Arc::new(OpenAiCompatible::new(
            reqwest::Client::new(),
            &settings.base_url,
            &settings.model,
            key,
        ))),
    };
    tracing::info!(provider = ?settings.provider, model = %settings.model, "triage ready");
    Triage::new(model, settings.timeout, settings.min_confidence)
}
