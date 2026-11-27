//! The store clients, with member prices applied for the household.

use std::sync::Arc;

use sqlx::PgPool;

use super::{apply, is_member};
use crate::services::stores::{BoxFuture, Product, Store, StoreClient, StoreClients, StoreError};

/// Wraps every client in `stores` so its products carry member prices only
/// when the household is a member at that store.
pub fn with_member_prices(stores: StoreClients, pool: PgPool) -> StoreClients {
    let wrapped = stores
        .iter()
        .map(|inner| {
            Arc::new(MemberPricedStore {
                inner: inner.clone(),
                pool: pool.clone(),
            }) as Arc<dyn StoreClient>
        })
        .collect();
    Arc::new(wrapped)
}

/// A [`StoreClient`] that prices each answer for the household's membership.
///
/// It sits outside the search cache, so a changed membership counts at once.
struct MemberPricedStore {
    inner: Arc<dyn StoreClient>,
    pool: PgPool,
}

impl StoreClient for MemberPricedStore {
    fn store(&self) -> Store {
        self.inner.store()
    }

    fn search<'a>(&'a self, query: &'a str) -> BoxFuture<'a, Result<Vec<Product>, StoreError>> {
        Box::pin(async move {
            let products = self.inner.search(query).await?;
            let member = is_member(&self.pool, self.store()).await;
            Ok(products.into_iter().map(|p| apply(p, member)).collect())
        })
    }

    fn product<'a>(
        &'a self,
        product_id: &'a str,
    ) -> BoxFuture<'a, Result<Option<Product>, StoreError>> {
        Box::pin(async move {
            let found = self.inner.product(product_id).await?;
            let member = is_member(&self.pool, self.store()).await;
            Ok(found.map(|p| apply(p, member)))
        })
    }
}
