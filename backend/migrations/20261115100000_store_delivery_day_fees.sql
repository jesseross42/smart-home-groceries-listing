-- The cheapest delivery fee each store showed for each day (issue #70).
--
-- The "Fill trolley" bookmark reads the store's delivery times while it
-- reserves one and reports the cheapest fee per day. The order planner uses
-- the cheapest fee on the household's delivery day (or on any day) instead of
-- the fee typed in Settings › Delivery. Past days are ignored, so old rows do
-- no harm; a new report for a day replaces the old one.
--
--   store          'woolworths' or 'coles'
--   delivery_date  the store-local day the times are on
--   cheapest_fee   the lowest fee of that day's open times, in dollars
--   reported_at    when the bookmark saw it

CREATE TABLE store_delivery_day_fees (
    store          TEXT        NOT NULL CHECK (store IN ('woolworths', 'coles')),
    delivery_date  DATE        NOT NULL,
    cheapest_fee   NUMERIC     NOT NULL CHECK (cheapest_fee >= 0),
    reported_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (store, delivery_date)
);
