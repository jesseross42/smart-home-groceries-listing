//! Store memberships: whether the household is in a store's rewards
//! program, so that store's member-only prices count.
//!
//! Only Woolworths has member prices (Everyday Rewards). A store whose
//! membership was never saved is not a member. For a non-member household
//! member prices and member-only multibuys are removed from every product
//! ([`apply`]), so the app never shows a price the household cannot get or
//! nudges it to join. [`with_member_prices`] wraps the store clients so the
//! search, the order review, the planner and purchase history all see the
//! same prices.
//!
//! SQL lives in [`repository`].

mod apply;
mod client;
mod log;
pub mod repository;

pub use apply::apply;
pub use client::with_member_prices;

use sqlx::PgPool;

use crate::error::ApiError;
use crate::services::stores::Store;

/// The rewards program whose members get `store`'s member prices, if any.
pub fn program(store: Store) -> Option<&'static str> {
    match store {
        Store::Woolworths => Some("Everyday Rewards"),
        Store::Coles => None,
    }
}

/// The household's membership at one store with a rewards program.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Membership {
    pub store: Store,
    pub member: bool,
}

/// Reads and saves the household's store memberships.
pub struct StoreMembershipsService<'a> {
    pool: &'a PgPool,
}

impl<'a> StoreMembershipsService<'a> {
    /// Borrows the pool for one request.
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Every store with a rewards program, in display order.
    pub async fn read(&self) -> Result<Vec<Membership>, ApiError> {
        let saved = repository::list(self.pool).await?;
        Ok(Store::ALL
            .into_iter()
            .filter(|store| program(*store).is_some())
            .map(|store| Membership {
                store,
                member: saved.iter().any(|(at, member)| *at == store && *member),
            })
            .collect())
    }

    /// Saves `changes`, answering with every membership as it now stands.
    /// 422 when a store has no rewards program. Saved together or not at all.
    pub async fn save(
        &self,
        changes: &[Membership],
        user_id: &str,
    ) -> Result<Vec<Membership>, ApiError> {
        if let Some(change) = changes.iter().find(|c| program(c.store).is_none()) {
            return Err(ApiError::UnprocessableEntity(format!(
                "{} has no member prices",
                change.store.display_name()
            )));
        }
        let mut tx = self.pool.begin().await?;
        for change in changes {
            repository::upsert(&mut *tx, change, user_id).await?;
        }
        tx.commit().await?;
        let saved = self.read().await?;
        log::saved(&saved, user_id);
        Ok(saved)
    }
}

/// Whether the household is a member at `store`. A store with no rewards
/// program is never asked about; a database error counts as not a member,
/// so a price is never lower than the household can really get.
pub async fn is_member(pool: &PgPool, store: Store) -> bool {
    if program(store).is_none() {
        return false;
    }
    match repository::find(pool, store).await {
        Ok(member) => member.unwrap_or(false),
        Err(err) => {
            log::read_failed(store, &err);
            false
        }
    }
}
