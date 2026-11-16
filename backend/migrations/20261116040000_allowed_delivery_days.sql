-- The weekdays the household can take a delivery on (Settings › Delivery,
-- issue #74).
--
-- order_preferences.allowed_delivery_days: an empty array = any day. The
-- order planner only costs delivery on these days and recommends nothing
-- when the delivery asked for cannot fall on one.

ALTER TABLE order_preferences
    ADD COLUMN allowed_delivery_days TEXT[] NOT NULL DEFAULT '{}'
        CHECK (allowed_delivery_days <@ ARRAY[
            'monday', 'tuesday', 'wednesday', 'thursday',
            'friday', 'saturday', 'sunday']);
