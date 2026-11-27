//! Every SQL statement for `store_memberships`.
//!
//! Every statement is a literal `&'static str` with bind parameters.

use sqlx::PgExecutor;

use super::Membership;
use crate::error::ApiError;
use crate::services::stores::Store;

/// Every saved membership. A store never saved has no row.
pub async fn list<'e, E>(executor: E) -> Result<Vec<(Store, bool)>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(
        sqlx::query_as::<_, (Store, bool)>("SELECT store, member FROM store_memberships")
            .fetch_all(executor)
            .await?,
    )
}

/// `store`'s saved membership, if it was ever saved.
pub async fn find<'e, E>(executor: E, store: Store) -> Result<Option<bool>, ApiError>
where
    E: PgExecutor<'e>,
{
    Ok(
        sqlx::query_scalar::<_, bool>("SELECT member FROM store_memberships WHERE store = $1")
            .bind(store)
            .fetch_optional(executor)
            .await?,
    )
}

/// Saves one store's membership, replacing what was there.
pub async fn upsert<'e, E>(
    executor: E,
    membership: &Membership,
    user_id: &str,
) -> Result<(), ApiError>
where
    E: PgExecutor<'e>,
{
    sqlx::query(
        "INSERT INTO store_memberships (store, member, updated_by)
         VALUES ($1, $2, $3)
         ON CONFLICT (store) DO UPDATE SET
             member = EXCLUDED.member,
             updated_by = EXCLUDED.updated_by,
             updated_at = now()",
    )
    .bind(membership.store)
    .bind(membership.member)
    .bind(user_id)
    .execute(executor)
    .await?;
    Ok(())
}
