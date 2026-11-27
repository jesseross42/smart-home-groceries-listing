//! Store memberships' log events. Nothing here is a secret.

use super::Membership;
use crate::error::ApiError;
use crate::services::stores::Store;

/// A household member saved the store memberships.
pub fn saved(memberships: &[Membership], user_id: &str) {
    let members: Vec<String> = memberships
        .iter()
        .map(|m| format!("{}: {}", m.store, m.member))
        .collect();
    tracing::info!(memberships = ?members, user_id, "store memberships saved");
}

/// The membership could not be read, so member prices are left out.
pub fn read_failed(store: Store, err: &ApiError) {
    tracing::warn!(%store, %err, "store membership could not be read; member prices left out");
}
