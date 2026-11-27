//! Bodies for Settings › Member prices: `GET` and
//! `PUT /api/store-memberships`.

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::services::store_memberships::{program, Membership};
use crate::services::stores::Store;

/// Every store with a rewards program, as `GET` and `PUT` answer.
#[derive(Debug, Serialize)]
pub struct StoreMembershipsResponse {
    pub stores: Vec<StoreMembershipResponse>,
}

/// One store's membership.
#[derive(Debug, Serialize)]
pub struct StoreMembershipResponse {
    pub store: Store,
    pub store_name: &'static str,
    /// The rewards program, e.g. `Everyday Rewards`.
    pub program: &'static str,
    pub member: bool,
}

/// `PUT` body: the memberships to change. Stores left out keep theirs.
#[derive(Debug, Deserialize, Validate)]
pub struct SaveStoreMembershipsBody {
    #[validate(length(min = 1, max = 2))]
    pub stores: Vec<StoreMembershipChange>,
}

/// One store's membership, as sent.
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreMembershipChange {
    pub store: Store,
    pub member: bool,
}

impl SaveStoreMembershipsBody {
    /// The changes, for the service.
    pub fn memberships(&self) -> Vec<Membership> {
        self.stores
            .iter()
            .map(|change| Membership {
                store: change.store,
                member: change.member,
            })
            .collect()
    }
}

impl From<Vec<Membership>> for StoreMembershipsResponse {
    fn from(memberships: Vec<Membership>) -> Self {
        Self {
            stores: memberships
                .into_iter()
                .map(|m| StoreMembershipResponse {
                    store: m.store,
                    store_name: m.store.display_name(),
                    program: program(m.store).unwrap_or_default(),
                    member: m.member,
                })
                .collect(),
        }
    }
}
