CREATE TABLE enrollments (
    id              UUID PRIMARY KEY,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    product_id      UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    cohort          TEXT,
    progress_pct    INTEGER NOT NULL DEFAULT 0 CHECK (progress_pct BETWEEN 0 AND 100),
    last_lesson_id  TEXT,
    source          TEXT NOT NULL CHECK (source IN ('purchase','subscription','manual')),
    source_ref_id   UUID,
    active          BOOLEAN NOT NULL DEFAULT TRUE,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at    TIMESTAMPTZ
);
CREATE UNIQUE INDEX enrollments_user_product_idx ON enrollments(user_id, product_id);
CREATE INDEX enrollments_user_idx ON enrollments(user_id);
