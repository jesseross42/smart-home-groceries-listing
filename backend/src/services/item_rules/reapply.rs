//! Putting one rule's chips on the items already on the list.
//!
//! Rules normally act only as an item arrives ([`super::apply`]). This is the
//! household asking for a new or edited rule to reach the items that were
//! there first, so it ignores `apply_to_manual`: the request itself is the
//! consent. Only items still under review change; committed items are locked.
//!
//! Each change carries the chips the item had before, which is what the web
//! app's Undo writes back through the ordinary item edit.

use sqlx::PgPool;
use uuid::Uuid;

use super::{log, matching, repository};
use crate::error::ApiError;
use crate::models::db::GroceryItem;
use crate::services::grocery::repository as items;
use crate::services::{filter_terms, selections};

/// One list item a rule added chips to.
#[derive(Debug)]
pub struct ReappliedItem {
    /// The item as it is now stored.
    pub item: GroceryItem,
    /// The chips it had before the rule was applied.
    pub previous_filter_terms: Vec<String>,
}

/// Adds the rule's filter terms to every active item its triggers match,
/// after the item's own chips. Returns the items that gained a chip, which is
/// empty when every match already had them. 404 when the rule is gone.
pub async fn apply_to_list(pool: &PgPool, rule_id: Uuid) -> Result<Vec<ReappliedItem>, ApiError> {
    let mut tx = pool.begin().await?;
    let rule = repository::find_rule(&mut *tx, rule_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(rule_id.to_string()))?;

    let mut changed = Vec::new();
    for before in items::lock_active_items(&mut tx).await? {
        if matching::matching_rules(&before.name, std::slice::from_ref(&rule)).is_empty() {
            continue;
        }
        let terms = filter_terms::merge(&before.filter_terms, &rule.filter_terms);
        if terms == before.filter_terms {
            continue;
        }
        let after = items::update_item(
            &mut tx,
            before.id,
            &before.name,
            before.quantity,
            before.note.as_deref(),
            &terms,
        )
        .await?;
        // New chips change the product search, so a product chosen under the
        // old chips no longer stands.
        selections::forget_if_stale(&mut tx, &before, &after).await?;
        changed.push(ReappliedItem {
            item: after,
            previous_filter_terms: before.filter_terms,
        });
    }
    tx.commit().await?;

    log::reapplied(&rule, &changed);
    Ok(changed)
}
