CREATE TABLE licenses (
    id                  UUID PRIMARY KEY,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id          UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    license_key_hash    BYTEA NOT NULL UNIQUE,
    license_key_prefix  TEXT NOT NULL,
    source              TEXT NOT NULL CHECK (source IN ('purchase','subscription','manual')),
    source_ref_id       UUID,
    issued_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at          TIMESTAMPTZ,
    active              BOOLEAN NOT NULL DEFAULT TRUE,
    download_count      INTEGER NOT NULL DEFAULT 0 CHECK (download_count >= 0),
    last_used_at        TIMESTAMPTZ,
    -- An inactive license must carry a revocation timestamp.
    CHECK (active OR revoked_at IS NOT NULL)
);
CREATE INDEX licenses_user_idx ON licenses(user_id, product_id) WHERE active = TRUE;
