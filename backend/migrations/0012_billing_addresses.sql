CREATE TABLE billing_addresses (
    user_id        UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    name           TEXT NOT NULL,
    company        TEXT,
    line1          TEXT NOT NULL,
    line2          TEXT,
    city           TEXT NOT NULL,
    state          TEXT,
    postal_code    TEXT NOT NULL,
    country        TEXT NOT NULL CHECK (length(country) = 2),
    tax_id         TEXT,
    billing_email  CITEXT NOT NULL,
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);
