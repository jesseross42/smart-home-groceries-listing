//! Bodies for removing or reducing a list item by voice, and undoing that.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use super::common::{MAX_NAME_LEN, MAX_QUANTITY};
use crate::models::db::{VoiceChangeKind, VoiceListChange};
use crate::services::voice_changes::RecentChange;

/// What the Alexa bridge forwards for "remove milk" or "remove two milk".
///
/// An absent `quantity` takes the whole item off the list. A present one
/// lowers the quantity by that much; reaching zero takes the item off too.
#[derive(Debug, Deserialize, Validate)]
pub struct AlexaRemoveCreate {
    #[validate(length(min = 1, max = MAX_NAME_LEN))]
    pub item: String,
    #[validate(range(min = 1, max = MAX_QUANTITY))]
    pub quantity: Option<i32>,
    /// Alexa's request id; a retried delivery with the same id changes nothing.
    #[validate(length(min = 1, max = 255))]
    pub external_id: Option<String>,
}

/// What the Alexa bridge forwards for "undo".
#[derive(Debug, Default, Deserialize, Validate)]
pub struct AlexaUndoCreate {
    /// Alexa's request id; a retried Undo with the same id undoes nothing more.
    #[validate(length(min = 1, max = 255))]
    pub external_id: Option<String>,
}

/// A voice change as the bridge sees it: enough to say what happened.
#[derive(Debug, Serialize)]
pub struct VoiceListChangeResponse {
    pub id: Uuid,
    pub kind: VoiceChangeKind,
    pub item_name: String,
    pub quantity_before: i32,
    /// `0` when the item was removed.
    pub quantity_after: i32,
    /// `true` once Undo has reversed the change.
    pub undone: bool,
}

impl From<VoiceListChange> for VoiceListChangeResponse {
    fn from(row: VoiceListChange) -> Self {
        Self {
            id: row.id,
            kind: row.kind,
            item_name: row.item_name,
            quantity_before: row.quantity_before,
            quantity_after: row.quantity_after,
            undone: row.undone_at.is_some(),
        }
    }
}

/// A voice change as the web app lists it: what happened, when, and whether
/// it can still be undone.
#[derive(Debug, Serialize)]
pub struct VoiceListChangeEntry {
    pub id: Uuid,
    pub kind: VoiceChangeKind,
    pub item_name: String,
    pub quantity_before: i32,
    /// `0` when the item was removed.
    pub quantity_after: i32,
    pub created_at: DateTime<Utc>,
    /// When Undo reversed it; `null` while it stands.
    pub undone_at: Option<DateTime<Utc>>,
    /// Still standing and young enough for Undo.
    pub can_undo: bool,
}

impl From<RecentChange> for VoiceListChangeEntry {
    fn from(recent: RecentChange) -> Self {
        let RecentChange { change, can_undo } = recent;
        Self {
            id: change.id,
            kind: change.kind,
            item_name: change.item_name,
            quantity_before: change.quantity_before,
            quantity_after: change.quantity_after,
            created_at: change.created_at,
            undone_at: change.undone_at,
            can_undo,
        }
    }
}
