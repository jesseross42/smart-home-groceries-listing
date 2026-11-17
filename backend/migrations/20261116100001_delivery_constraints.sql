-- Household deadlines and item-specific freshness, plus replaceable store snapshots.
ALTER TABLE order_preferences
    ADD COLUMN need_by TIMESTAMP,
    ADD COLUMN perishable_hours SMALLINT CHECK (
        perishable_hours IS NULL OR (perishable_hours BETWEEN 1 AND 168 AND need_by IS NOT NULL)
    );

ALTER TABLE grocery_items ADD COLUMN perishable BOOLEAN NOT NULL DEFAULT false;
ALTER TABLE voice_list_changes ADD COLUMN item_perishable BOOLEAN NOT NULL DEFAULT false;

CREATE TABLE store_delivery_window_snapshots (
    store TEXT PRIMARY KEY CHECK (store IN ('woolworths', 'coles')),
    windows JSONB NOT NULL CHECK (jsonb_typeof(windows) = 'array'),
    reported_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
