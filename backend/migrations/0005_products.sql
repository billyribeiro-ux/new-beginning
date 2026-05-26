CREATE TABLE products (
    id                    UUID PRIMARY KEY,
    slug                  TEXT NOT NULL UNIQUE,
    legacy_slug_id        TEXT UNIQUE,
    kind                  TEXT NOT NULL CHECK (kind IN ('indicator','course')),
    name                  TEXT NOT NULL,
    tagline               TEXT NOT NULL,
    description           TEXT NOT NULL,
    price_cents           BIGINT NOT NULL CHECK (price_cents >= 0),
    original_price_cents  BIGINT CHECK (original_price_cents IS NULL OR original_price_cents >= 0),
    active                BOOLEAN NOT NULL DEFAULT TRUE,
    badge                 TEXT,
    rating_value          NUMERIC(3,2) NOT NULL DEFAULT 0.00 CHECK (rating_value BETWEEN 0 AND 5),
    rating_count          INTEGER NOT NULL DEFAULT 0 CHECK (rating_count >= 0),
    highlights            TEXT[] NOT NULL DEFAULT '{}',
    specs_json            JSONB NOT NULL DEFAULT '[]'::jsonb,
    deliverables          TEXT[] NOT NULL DEFAULT '{}',
    requirements          TEXT[] NOT NULL DEFAULT '{}',
    media_poster_color    TEXT NOT NULL,
    media_accent          TEXT NOT NULL,
    stripe_price_id       TEXT UNIQUE,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX products_slug_idx        ON products(slug);
CREATE INDEX products_active_kind_idx ON products(kind, active) WHERE active = TRUE;
CREATE INDEX products_stripe_price_id_idx ON products(stripe_price_id) WHERE stripe_price_id IS NOT NULL;
