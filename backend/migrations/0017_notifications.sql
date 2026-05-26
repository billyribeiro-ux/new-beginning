CREATE TABLE notification_preferences (
    user_id                UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    product_updates_email  BOOLEAN NOT NULL DEFAULT TRUE,
    product_updates_inapp  BOOLEAN NOT NULL DEFAULT TRUE,
    billing_email          BOOLEAN NOT NULL DEFAULT TRUE,
    billing_inapp          BOOLEAN NOT NULL DEFAULT TRUE,
    course_progress_email  BOOLEAN NOT NULL DEFAULT TRUE,
    course_progress_inapp  BOOLEAN NOT NULL DEFAULT TRUE,
    market_alerts_email    BOOLEAN NOT NULL DEFAULT FALSE,
    market_alerts_inapp    BOOLEAN NOT NULL DEFAULT TRUE,
    marketing_email        BOOLEAN NOT NULL DEFAULT FALSE,
    marketing_inapp        BOOLEAN NOT NULL DEFAULT FALSE,
    dnd_enabled            BOOLEAN NOT NULL DEFAULT FALSE,
    dnd_start              TIME,
    dnd_end                TIME,
    timezone               TEXT NOT NULL DEFAULT 'UTC',
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((dnd_enabled = FALSE) OR (dnd_start IS NOT NULL AND dnd_end IS NOT NULL))
);

CREATE TABLE notifications (
    id          UUID PRIMARY KEY,
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL,
    title       TEXT NOT NULL,
    body        TEXT NOT NULL,
    source      TEXT NOT NULL,
    read_at     TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX notifications_user_unread_idx
    ON notifications(user_id, created_at DESC) WHERE read_at IS NULL;
CREATE INDEX notifications_user_all_idx ON notifications(user_id, created_at DESC);
