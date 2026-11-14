-- A saved default delivery day for trolley handoffs
-- (docs/features/FEATURE_TROLLEY_HANDOFF.md).
--
-- order_preferences.default_delivery_day: the weekday the Send to store
-- sheet picks first; NULL = no preference (the sheet picks tomorrow). It is
-- only a starting choice: each handoff still carries its own date.

ALTER TABLE order_preferences
    ADD COLUMN default_delivery_day TEXT
        CHECK (default_delivery_day IN (
            'monday', 'tuesday', 'wednesday', 'thursday', 'friday',
            'saturday', 'sunday'));
