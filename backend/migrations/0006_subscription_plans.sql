CREATE TABLE subscription_plans (
    id                        UUID PRIMARY KEY,
    slug                      TEXT NOT NULL UNIQUE,
    legacy_slug_id            TEXT UNIQUE,
    name                      TEXT NOT NULL,
    cadence                   TEXT NOT NULL CHECK (cadence IN ('monthly','quarterly','annual')),
    price_cents               BIGINT NOT NULL CHECK (price_cents >= 0),
    monthly_equivalent_cents  BIGINT NOT NULL CHECK (monthly_equivalent_cents >= 0),
    savings_pct               INTEGER NOT NULL DEFAULT 0 CHECK (savings_pct BETWEEN 0 AND 100),
    tagline                   TEXT NOT NULL,
    highlights                TEXT[] NOT NULL DEFAULT '{}',
    featured                  BOOLEAN NOT NULL DEFAULT FALSE,
    badge                     TEXT,
    stripe_price_id           TEXT NOT NULL UNIQUE,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);
-- Intent: exactly ONE featured plan across the whole pricing page (the single
-- "most popular" callout in the locked Phase-1 UI). This is deliberate and
-- global, NOT one-featured-per-cadence. If the UI ever wants a highlight per
-- cadence column, this index must change to ON (cadence) WHERE featured.
CREATE UNIQUE INDEX subscription_plans_one_featured_idx
    ON subscription_plans((TRUE)) WHERE featured = TRUE;
