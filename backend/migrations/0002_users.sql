CREATE TABLE users (
    id                       UUID PRIMARY KEY,
    email                    CITEXT NOT NULL UNIQUE,
    name                     TEXT NOT NULL,
    headline                 TEXT,
    timezone                 TEXT NOT NULL DEFAULT 'UTC',
    language                 TEXT NOT NULL DEFAULT 'en',
    password_hash            TEXT,
    role                     TEXT NOT NULL DEFAULT 'member'
                                CHECK (role IN ('member','admin')),
    email_verified_at        TIMESTAMPTZ,
    stripe_customer_id       TEXT UNIQUE,
    totp_secret_encrypted    BYTEA,
    totp_enabled_at          TIMESTAMPTZ,
    twofa_backup_codes_hash  BYTEA[],
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at               TIMESTAMPTZ
);
CREATE INDEX users_deleted_at_idx ON users(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX users_stripe_customer_id_idx ON users(stripe_customer_id) WHERE stripe_customer_id IS NOT NULL;
