CREATE TABLE email_verifications (
    id           UUID PRIMARY KEY,
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash   BYTEA NOT NULL UNIQUE,
    kind         TEXT NOT NULL CHECK (kind IN ('signup','email_change','password_reset')),
    new_email    CITEXT,
    expires_at   TIMESTAMPTZ NOT NULL,
    consumed_at  TIMESTAMPTZ,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK ((kind = 'email_change') = (new_email IS NOT NULL))
);
CREATE INDEX email_ver_user_pending_idx ON email_verifications(user_id, kind)
    WHERE consumed_at IS NULL;
