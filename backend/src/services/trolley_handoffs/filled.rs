//! The `trolley_filled` push for a reported handoff. Pure.

use uuid::Uuid;

use crate::models::trolley_handoff_rows::{TrolleyHandoffLine, TrolleyLineOutcome};
use crate::services::stores::Store;
use crate::services::ws_hub::ServerEvent;

/// The event telling open sessions that `handoff_id`'s trolley at `store`
/// was filled, counting its reported `lines`.
pub fn filled_event(handoff_id: Uuid, store: Store, lines: &[TrolleyHandoffLine]) -> ServerEvent {
    let count = |outcome| lines.iter().filter(|l| l.outcome == Some(outcome)).count();
    ServerEvent::TrolleyFilled {
        handoff_id,
        store,
        store_name: store.display_name(),
        added: count(TrolleyLineOutcome::Added),
        not_added: count(TrolleyLineOutcome::Failed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(outcome: Option<TrolleyLineOutcome>) -> TrolleyHandoffLine {
        TrolleyHandoffLine {
            handoff_id: Uuid::nil(),
            grocery_item_id: Uuid::new_v4(),
            product_id: "1".into(),
            product_name: "Milk".into(),
            quantity: 1,
            outcome,
            problem: None,
        }
    }

    #[test]
    fn counts_what_went_in_and_what_did_not() {
        let lines = [
            line(Some(TrolleyLineOutcome::Added)),
            line(Some(TrolleyLineOutcome::Added)),
            line(Some(TrolleyLineOutcome::Failed)),
        ];
        assert_eq!(
            filled_event(Uuid::nil(), Store::Woolworths, &lines),
            ServerEvent::TrolleyFilled {
                handoff_id: Uuid::nil(),
                store: Store::Woolworths,
                store_name: "Woolworths",
                added: 2,
                not_added: 1,
            }
        );
    }
}
