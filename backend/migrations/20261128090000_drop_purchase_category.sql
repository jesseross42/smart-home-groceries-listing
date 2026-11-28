-- Drops the two purchase columns that migration 20261127090000 retired.
-- `bought_products` owns each product's category (migration 20261119090000
-- copied every purchase's value there), and no query reads or writes these.
--
-- This removes data, so the production update applies it only as a planned
-- removal: this file's SHA-256 is listed in `scripts/planned-removals.json`.
-- Editing this file, even a comment, withdraws that approval.

ALTER TABLE purchases
    DROP COLUMN category,
    DROP COLUMN category_source;
