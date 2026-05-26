CREATE TABLE order_items (
    id                UUID PRIMARY KEY,
    order_id          UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id        UUID REFERENCES products(id) ON DELETE RESTRICT,
    plan_id           UUID REFERENCES subscription_plans(id) ON DELETE RESTRICT,
    quantity          INTEGER NOT NULL DEFAULT 1 CHECK (quantity > 0),
    unit_price_cents  BIGINT NOT NULL CHECK (unit_price_cents >= 0),
    line_total_cents  BIGINT NOT NULL CHECK (line_total_cents >= 0),
    name_snapshot     TEXT NOT NULL,
    slug_snapshot     TEXT NOT NULL,
    CHECK ((product_id IS NOT NULL) <> (plan_id IS NOT NULL))
);
CREATE INDEX order_items_order_id_idx ON order_items(order_id);
