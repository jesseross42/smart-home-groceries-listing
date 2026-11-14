//! Every SQL statement for `openai_triage_key`.
//!
//! Every statement is a literal `&'static str` with bind parameters.

use sqlx::PgExecutor;

use crate::error::ApiError;
use crate::models::openai_key_rows::OpenAiKeyRow;

/// The saved row, if a key was ever saved.
pub async fn find<'e, E>(executor: E) -> Result<Option<OpenAiKeyRow>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(sqlx::query_as::<_, OpenAiKeyRow>(
        "SELECT api_key_encrypted, previous_api_key_encrypted, updated_by, updated_at
         FROM openai_triage_key",
    )
    .fetch_optional(executor)
    .await?)
}

/// Makes `api_key_encrypted` the saved key (`None` removes it) and keeps
/// what it replaced as the previous one.
pub async fn replace<'e, E>(
    executor: E,
    api_key_encrypted: Option<&str>,
    user_id: &str,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO openai_triage_key (api_key_encrypted, updated_by)
         VALUES ($1, $2)
         ON CONFLICT (singleton) DO UPDATE SET
             previous_api_key_encrypted = openai_triage_key.api_key_encrypted,
             api_key_encrypted = EXCLUDED.api_key_encrypted,
             updated_by = EXCLUDED.updated_by,
             updated_at = now()",
    )
    .bind(api_key_encrypted)
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(())
}

/// Swaps the saved key and the previous one: undoes the last save, removal
/// or undo. Does nothing when no key was ever saved.
pub async fn swap_with_previous<'e, E>(executor: E, user_id: &str) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "UPDATE openai_triage_key SET
             api_key_encrypted = previous_api_key_encrypted,
             previous_api_key_encrypted = api_key_encrypted,
             updated_by = $1,
             updated_at = now()",
    )
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(())
}
