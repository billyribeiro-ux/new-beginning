-- Two concerns, deliberately kept separable on one table:
--   (a) idempotency  -> event_id PK + ON CONFLICT DO NOTHING
--   (b) lifecycle    -> processed_at lets a received-but-unprocessed event
--                       be found and re-driven; a crash between insert and
--                       dispatch is no longer invisible.
-- payload is for replay/debug ONLY and is NOT load-bearing for dedupe. It is
-- nullable so a retention job can null it out without losing the dedupe row.
CREATE TABLE stripe_events (
    event_id          TEXT PRIMARY KEY,         -- Stripe's evt_… id IS the PK
    event_type        TEXT NOT NULL,
    payload           JSONB,                    -- nullable: replay/debug only
    received_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at      TIMESTAMPTZ,
    attempts          INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    processing_error  TEXT
);
-- Drives the reconciliation sweep (§8.3): everything received but not processed.
CREATE INDEX stripe_events_unprocessed_idx ON stripe_events(received_at)
    WHERE processed_at IS NULL;
CREATE INDEX stripe_events_kind_idx ON stripe_events(event_type, received_at DESC);
