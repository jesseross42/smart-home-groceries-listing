-- The notes a person wrote on a Google Tasks task (issue #86).
--
-- voice_requests.source_note: shown on the request's Pending card for a
-- person to read. Never parsed: the item name and quantity still come only
-- from the task's title. NULL for every other channel and for a task with no
-- notes.

ALTER TABLE voice_requests
    ADD COLUMN source_note TEXT CHECK (char_length(source_note) <= 2000);
