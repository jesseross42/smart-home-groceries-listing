-- Retention for access_logs (docs/features/FEATURE_ACCESS_LOGS.md).
--
-- The daily clean-up deletes rows by age. Without this index every run reads
-- the whole table to find the old ones.

CREATE INDEX ix_access_logs_occurred_at ON access_logs (occurred_at);
