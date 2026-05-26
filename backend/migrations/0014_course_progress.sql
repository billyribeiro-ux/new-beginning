CREATE TABLE course_progress (
    id                  UUID PRIMARY KEY,
    enrollment_id       UUID NOT NULL REFERENCES enrollments(id) ON DELETE CASCADE,
    lesson_id           TEXT NOT NULL,
    completed_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    notes               TEXT NOT NULL DEFAULT '',
    time_spent_seconds  INTEGER NOT NULL DEFAULT 0 CHECK (time_spent_seconds >= 0),
    payload             JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE UNIQUE INDEX course_progress_enrollment_lesson_idx
    ON course_progress(enrollment_id, lesson_id);
