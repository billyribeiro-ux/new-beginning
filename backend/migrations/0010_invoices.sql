CREATE TABLE invoices (
    id                 UUID PRIMARY KEY,
    order_id           UUID REFERENCES orders(id) ON DELETE RESTRICT,
    user_id            UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    stripe_invoice_id  TEXT NOT NULL UNIQUE,
    number             TEXT NOT NULL,
    status             TEXT NOT NULL CHECK (status IN ('draft','open','paid','void','uncollectible')),
    amount_cents       BIGINT NOT NULL CHECK (amount_cents >= 0),
    currency           TEXT NOT NULL DEFAULT 'usd',
    invoice_date       TIMESTAMPTZ NOT NULL,
    pdf_r2_key         TEXT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX invoices_user_date_idx ON invoices(user_id, invoice_date DESC);
