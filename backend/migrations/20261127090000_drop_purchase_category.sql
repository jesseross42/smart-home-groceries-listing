-- Purchases stop carrying a category. `bought_products` (migration
-- 20261119090000) owns each product's category, and the spending analysis
-- reads it from there. The copy on `purchases` was written on every save but
-- never read, and "Sort categories again" no longer updated it, so it went
-- stale. Totals are untouched: no other column changes.

ALTER TABLE purchases
    DROP COLUMN category,
    DROP COLUMN category_source;
