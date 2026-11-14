-- The household's usual delivery day (Settings › Delivery).
--
-- order_preferences.default_delivery_day: the weekday the delivery chooser
-- starts on when a list is sent to a store. NULL = no usual day, so the
-- chooser starts on tomorrow. The household can pick another day each shop.

ALTER TABLE order_preferences
    ADD COLUMN default_delivery_day TEXT CHECK (default_delivery_day IN (
        'monday', 'tuesday', 'wednesday', 'thursday',
        'friday', 'saturday', 'sunday'));
