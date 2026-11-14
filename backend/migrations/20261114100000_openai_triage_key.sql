-- The OpenAI triage key, saved from Settings › Help.
--
-- openai_triage_key: a single row once a key was ever saved.
--   api_key_encrypted           AES-256-GCM (services/encryption.rs);
--                               NULL = no saved key (removed)
--   previous_api_key_encrypted  what the last save or removal replaced, kept
--                               only so that change can be undone
--
-- The key is never returned to the browser and never logged.

CREATE TABLE openai_triage_key (
    singleton                  BOOLEAN     PRIMARY KEY DEFAULT true CHECK (singleton),
    api_key_encrypted          TEXT        CHECK (api_key_encrypted IS NULL
                                                  OR char_length(api_key_encrypted) <= 4000),
    previous_api_key_encrypted TEXT        CHECK (previous_api_key_encrypted IS NULL
                                                  OR char_length(previous_api_key_encrypted) <= 4000),
    updated_by                 TEXT        NOT NULL CHECK (char_length(updated_by) BETWEEN 1 AND 255),
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now()
);
