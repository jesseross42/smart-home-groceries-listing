-- Purchases stop carrying a category. `bought_products` (migration
-- 20261119090000) owns each product's category, and the spending analysis
-- reads it from there. The copy on `purchases` was written on every save but
-- never read, and "Sort categories again" no longer updated it, so it went
-- stale.
--
-- The columns are retired, not dropped: the production installer refuses
-- destructive migrations (docs/production.md), so removing them is planned
-- separately. New purchases leave them NULL, and no query reads them.

ALTER TABLE purchases
    ALTER COLUMN category DROP NOT NULL,
    ALTER COLUMN category_source DROP NOT NULL;

COMMENT ON COLUMN purchases.category IS
    'Retired 2026-11-27: read bought_products.category. NULL on new rows.';
COMMENT ON COLUMN purchases.category_source IS
    'Retired 2026-11-27: read bought_products.category_source. NULL on new rows.';
