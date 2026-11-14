-- The address each store delivers to (Settings › Delivery).
--
-- store_delivery_fees.delivery_address: one of the addresses already saved
-- in the household's account on the store's website, as the website writes
-- it (e.g. "U 2 8 Athelstan Rd, CAMBERWELL VIC 3124"). The store tab picks
-- the matching saved address before choosing a delivery time. NULL = keep
-- whatever address the account has.

ALTER TABLE store_delivery_fees
    ADD COLUMN delivery_address TEXT
        CHECK (char_length(delivery_address) BETWEEN 1 AND 200);
