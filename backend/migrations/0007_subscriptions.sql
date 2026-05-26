CREATE TABLE subscriptions (
    id                         UUID PRIMARY KEY,
    user_id                    UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    plan_id                    UUID NOT NULL REFERENCES subscription_plans(id) ON DELETE RESTRICT,
    stripe_subscription_id     TEXT NOT NULL UNIQUE,
    status                     TEXT NOT NULL CHECK (status IN
                                 ('trialing','active','past_due','paused','scheduled_cancel','canceled')),
    cancel_at_period_end       BOOLEAN NOT NULL DEFAULT FALSE,
    current_period_start       TIMESTAMPTZ NOT NULL,
    current_period_end         TIMESTAMPTZ NOT NULL,
    pause_until                TIMESTAMPTZ,
    canceled_at                TIMESTAMPTZ,
    default_payment_method_id  TEXT,
    created_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    CHECK (current_period_end > current_period_start),
    CHECK ((status = 'canceled') = (canceled_at IS NOT NULL))
);
CREATE INDEX subscriptions_user_status_idx     ON subscriptions(user_id, status);
CREATE INDEX subscriptions_stripe_sub_id_idx   ON subscriptions(stripe_subscription_id);
CREATE INDEX subscriptions_period_end_idx      ON subscriptions(current_period_end)
    WHERE status IN ('active','trialing','past_due');
