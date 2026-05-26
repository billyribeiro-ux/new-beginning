CREATE TABLE audit_log (
    id             UUID PRIMARY KEY,
    actor_user_id  UUID REFERENCES users(id) ON DELETE SET NULL,
    action         TEXT NOT NULL,
    target_kind    TEXT NOT NULL,
    target_id      TEXT NOT NULL,
    metadata       JSONB NOT NULL DEFAULT '{}'::jsonb,
    ip             INET,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX audit_log_actor_created_idx ON audit_log(actor_user_id, created_at DESC)
    WHERE actor_user_id IS NOT NULL;
CREATE INDEX audit_log_target_idx ON audit_log(target_kind, target_id, created_at DESC);
