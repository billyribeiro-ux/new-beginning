CREATE TABLE downloads_catalog (
    id           UUID PRIMARY KEY,
    product_id   UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    platform     TEXT NOT NULL,
    version      TEXT NOT NULL,
    file_r2_key  TEXT NOT NULL,
    sha256       TEXT NOT NULL CHECK (length(sha256) = 64),
    size_bytes   BIGINT NOT NULL CHECK (size_bytes > 0),
    released_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX downloads_catalog_product_idx ON downloads_catalog(product_id, released_at DESC);

CREATE TABLE download_grants (
    id                  UUID PRIMARY KEY,
    user_id             UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    download_id         UUID NOT NULL REFERENCES downloads_catalog(id) ON DELETE CASCADE,
    granted_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_downloaded_at  TIMESTAMPTZ,
    download_count      INTEGER NOT NULL DEFAULT 0 CHECK (download_count >= 0),
    UNIQUE (user_id, download_id)
);
CREATE INDEX download_grants_user_idx ON download_grants(user_id);
