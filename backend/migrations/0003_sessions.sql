CREATE TABLE sessions (
    id            UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash    BYTEA NOT NULL UNIQUE,
    user_agent    TEXT,
    ip            INET,
    last_seen_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at    TIMESTAMPTZ NOT NULL,
    revoked_at    TIMESTAMPTZ
);
CREATE INDEX sessions_token_hash_idx   ON sessions(token_hash);
CREATE INDEX sessions_user_active_idx  ON sessions(user_id) WHERE revoked_at IS NULL;
CREATE INDEX sessions_expires_at_idx   ON sessions(expires_at) WHERE revoked_at IS NULL;
