-- Bought products: one row per store product the household has bought, with
-- the category the spending analysis files it under.
--
-- Before this, each purchase row carried its own copy of the product's
-- details and category, so "Sort categories again" had to rewrite every
-- purchase, and two purchases of one product could sit in two categories.
-- Now the analysis reads a product's category from here; `purchases` keeps
-- its columns as the snapshot of what was bought that day.
--
-- Owner: the purchase history (`services/purchases/products.rs`). Nothing
-- else writes this table.
--
-- Refresh policy:
--   * saving a shop writes each product's details from that purchase, unless
--     a newer purchase of the product is already saved (`refreshed_at`);
--   * Undo of a shop refreshes each of its products from the newest purchase
--     left, and deletes the row when none is left;
--   * "Sort categories again" works out every row's category again.
--
-- Columns
--   product_name, brand, package_size, store_category
--                   as the store gave them on the newest purchase
--   category        the category the analysis shows (category.rs rules)
--   category_source store, name or none, as on `purchases`
--   refreshed_at    the `bought_at` of the purchase the details came from

CREATE TABLE bought_products (
    store            TEXT         NOT NULL CHECK (store IN ('woolworths', 'coles')),
    product_id       TEXT         NOT NULL CHECK (char_length(product_id) BETWEEN 1 AND 100),
    product_name     TEXT         NOT NULL CHECK (char_length(product_name) BETWEEN 1 AND 300),
    brand            TEXT         CHECK (brand IS NULL OR char_length(brand) <= 200),
    package_size     TEXT         CHECK (package_size IS NULL OR char_length(package_size) <= 100),
    store_category   TEXT         CHECK (store_category IS NULL
                                         OR char_length(store_category) <= 200),
    category         TEXT         NOT NULL CHECK (char_length(category) BETWEEN 1 AND 100),
    category_source  TEXT         NOT NULL CHECK (category_source IN ('store', 'name', 'none')),
    refreshed_at     TIMESTAMPTZ  NOT NULL,
    PRIMARY KEY (store, product_id)
);

-- The category filter and the category list read by category.
CREATE INDEX ix_bought_products_category ON bought_products (category);

-- Backfill: every product bought so far, from its newest purchase, keeping
-- the category that purchase was given. Totals are untouched: no purchase
-- row changes.
INSERT INTO bought_products
    (store, product_id, product_name, brand, package_size, store_category,
     category, category_source, refreshed_at)
SELECT DISTINCT ON (store, product_id)
       store, product_id, product_name, brand, package_size, store_category,
       category, category_source, bought_at
FROM purchases
ORDER BY store, product_id, bought_at DESC, created_at DESC, id DESC;

-- Every purchase has its product row, so the analysis can join without
-- losing a purchase (and so without changing a total).
ALTER TABLE purchases
    ADD CONSTRAINT fk_purchases_bought_product
    FOREIGN KEY (store, product_id) REFERENCES bought_products (store, product_id);
