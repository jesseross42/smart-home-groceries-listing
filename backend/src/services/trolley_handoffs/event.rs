//! The real-time event for a filled trolley
//! (`docs/features/FEATURE_DESKTOP_APP.md`).

use super::HandoffWithLines;
use crate::models::db::TrolleyLineOutcome;
use crate::services::ws_hub::ServerEvent;

/// The event announcing that `closed`'s store tab has reported: the store
/// and how many products went in or were refused.
pub fn filled(closed: &HandoffWithLines) -> ServerEvent {
    let count = |outcome: TrolleyLineOutcome| {
        closed
            .lines
            .iter()
            .filter(|line| line.outcome == Some(outcome))
            .count()
    };
    ServerEvent::TrolleyFilled {
        handoff_id: closed.handoff.id,
        store_name: closed.handoff.store.display_name().to_string(),
        added: count(TrolleyLineOutcome::Added),
        failed: count(TrolleyLineOutcome::Failed),
    }
}
