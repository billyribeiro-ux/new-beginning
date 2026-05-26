CREATE TABLE payment_methods (
    id                        UUID PRIMARY KEY,
    user_id                   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    stripe_payment_method_id  TEXT NOT NULL UNIQUE,
    brand                     TEXT NOT NULL,
    last4                     TEXT NOT NULL CHECK (length(last4) = 4),
    exp_month                 INTEGER NOT NULL CHECK (exp_month BETWEEN 1 AND 12),
    exp_year                  INTEGER NOT NULL CHECK (exp_year BETWEEN 2024 AND 2099),
    is_default                BOOLEAN NOT NULL DEFAULT FALSE,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE UNIQUE INDEX payment_methods_one_default_per_user
    ON payment_methods(user_id) WHERE is_default = TRUE;
CREATE INDEX payment_methods_user_idx ON payment_methods(user_id);
