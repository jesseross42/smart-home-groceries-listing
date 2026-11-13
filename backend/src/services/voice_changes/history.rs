//! The household's view of voice changes: the newest ones, each with
//! whether Undo can still reach it.

use chrono::Utc;
use sqlx::PgPool;

use super::{repository, undo};
use crate::error::ApiError;
use crate::models::db::VoiceListChange;

/// How many changes the web app lists.
const RECENT_LIMIT: i64 = 50;

/// One listed change and whether it can still be undone.
#[derive(Debug)]
pub struct RecentChange {
    pub change: VoiceListChange,
    /// Still standing and inside the Undo window.
    pub can_undo: bool,
}

/// The newest voice changes from every source, newest first.
pub async fn recent(pool: &PgPool) -> Result<Vec<RecentChange>, ApiError> {
    let now = Utc::now();
    let changes = repository::list_recent(pool, RECENT_LIMIT).await?;
    Ok(changes
        .into_iter()
        .map(|change| RecentChange {
            can_undo: change.undone_at.is_none() && undo::within_window(&change, now),
            change,
        })
        .collect())
}
