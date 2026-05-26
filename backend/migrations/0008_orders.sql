-- `total = subtotal + tax` is enforced in code (OrdersRepo::create), not as
-- a CHECK, so refunds-with-fees and credit-note scenarios don't fight the DB.
CREATE TABLE orders (
    id                          UUID PRIMARY KEY,
    user_id                     UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    status                      TEXT NOT NULL CHECK (status IN ('pending','paid','refunded','failed')),
    subtotal_cents              BIGINT NOT NULL CHECK (subtotal_cents >= 0),
    tax_cents                   BIGINT NOT NULL DEFAULT 0 CHECK (tax_cents >= 0),
    total_cents                 BIGINT NOT NULL CHECK (total_cents >= 0),
    currency                    TEXT NOT NULL DEFAULT 'usd',
    stripe_checkout_session_id  TEXT UNIQUE,
    stripe_payment_intent_id    TEXT UNIQUE,
    paid_at                     TIMESTAMPTZ,
    refunded_at                 TIMESTAMPTZ,
    cart_snapshot               JSONB NOT NULL DEFAULT '[]'::jsonb,
    expires_at                  TIMESTAMPTZ,
    created_at                  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX orders_user_created_idx ON orders(user_id, created_at DESC);
CREATE INDEX orders_status_created_idx ON orders(status, created_at DESC)
    WHERE status IN ('pending','failed');
CREATE INDEX orders_stripe_pi_idx ON orders(stripe_payment_intent_id)
    WHERE stripe_payment_intent_id IS NOT NULL;
