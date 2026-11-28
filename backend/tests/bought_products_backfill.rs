//! Migration `20261119090000_bought_products` on a database that already
//! holds purchases: each product bought gets one record, from its newest
//! purchase, and every purchase stays readable with its total unchanged.
//! Migration `20261128090000_drop_purchase_category` then drops the
//! purchases' own copy of the category.

use std::borrow::Cow;

use grocery_backend::db::MIGRATOR;
use sqlx::migrate::Migrator;
use sqlx::PgPool;
use uuid::Uuid;

/// The version of the migration under test.
const BOUGHT_PRODUCTS: i64 = 20261119090000;

/// Applies every migration before [`BOUGHT_PRODUCTS`].
async fn migrate_to_before(pool: &PgPool) {
    let before = Migrator {
        migrations: Cow::Owned(
            MIGRATOR
                .migrations
                .iter()
                .filter(|migration| migration.version < BOUGHT_PRODUCTS)
                .cloned()
                .collect(),
        ),
        ..Migrator::DEFAULT
    };
    before.run(pool).await.unwrap();
}

/// Writes one purchase of `product_id` as it was saved before the migration.
async fn old_purchase(pool: &PgPool, product_id: &str, name: &str, category: &str, at: &str) {
    let order_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO purchase_orders (id, store, items_total, recorded_by, bought_at)
         VALUES ($1, 'coles', 2, 'dev-user', $2::timestamptz)",
    )
    .bind(order_id)
    .bind(at)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO purchases (id, order_id, item_name, item_key, item_status_before, store,
                                product_id, product_name, quantity, unit_price, total_price,
                                category, category_source, bought_at)
         VALUES ($1, $2, 'milk', 'milk', 'active', 'coles', $3, $4, 1, 2, 2, $5, 'name',
                 $6::timestamptz)",
    )
    .bind(Uuid::new_v4())
    .bind(order_id)
    .bind(product_id)
    .bind(name)
    .bind(category)
    .bind(at)
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrations = false)]
async fn existing_purchases_are_backfilled_from_the_newest(pool: PgPool) {
    migrate_to_before(&pool).await;
    old_purchase(&pool, "c-milk", "Milk 2L", "Other", "2026-07-01T00:00:00Z").await;
    old_purchase(
        &pool,
        "c-milk",
        "Milk 2L New",
        "Dairy & eggs",
        "2026-08-01T00:00:00Z",
    )
    .await;
    old_purchase(&pool, "c-bread", "Bread", "Bakery", "2026-07-15T00:00:00Z").await;

    MIGRATOR.run(&pool).await.unwrap();

    let records: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT product_id, product_name, category FROM bought_products ORDER BY product_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        records,
        vec![
            ("c-bread".into(), "Bread".into(), "Bakery".into()),
            ("c-milk".into(), "Milk 2L New".into(), "Dairy & eggs".into()),
        ]
    );
    let (count, total): (i64, rust_decimal::Decimal) =
        sqlx::query_as("SELECT count(*), sum(total_price) FROM purchases")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((count, total), (3, rust_decimal::Decimal::from(6)));
    let own_category_columns: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.columns
         WHERE table_schema = current_schema() AND table_name = 'purchases'
           AND column_name IN ('category', 'category_source')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(own_category_columns, 0);
}
