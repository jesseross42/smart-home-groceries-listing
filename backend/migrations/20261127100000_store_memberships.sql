-- Store memberships: whether the household is a member of a store's rewards
-- programme (Woolworths Everyday Rewards), so member-only prices count.
--
-- Owner: `services/store_memberships`. A store with no row is not a member.
-- Without a row, member prices and member-only multibuys are hidden, so the
-- app never shows a price the household cannot get (and never invites it to
-- join a programme).
--
-- Columns
--   member      the household's account at this store is a member
--   updated_by  the signed-in user who last saved it

CREATE TABLE store_memberships (
    store       TEXT         PRIMARY KEY CHECK (store IN ('woolworths', 'coles')),
    member      BOOLEAN      NOT NULL,
    updated_by  TEXT         NOT NULL,
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT now()
);
