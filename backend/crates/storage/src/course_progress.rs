//! `CourseProgressRepo` — per-lesson completion state.
//!
//! BACKEND.md §4 0014. One row per `(enrollment_id, lesson_id)`. UPSERTs
//! on the unique index — re-marking a lesson complete is a no-op (the
//! `completed_at` stays the original time; only `time_spent_seconds`,
//! `notes`, and `payload` move).
//!
//! The `lesson_id` is a free-form TEXT string identifying a lesson in the
//! course's static manifest (e.g. `"mod-3.lesson-2"`). The api crate
//! validates it against the course's manifest before writing.

use common::ids::EnrollmentId;
use serde_json::Value as Json;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct LessonProgress {
    pub id: Uuid,
    pub enrollment_id: EnrollmentId,
    pub lesson_id: String,
    pub completed_at: OffsetDateTime,
    pub notes: String,
    pub time_spent_seconds: i32,
    pub payload: Json,
}

#[derive(Clone)]
pub struct CourseProgressRepo {
    pool: PgPool,
}

#[derive(Debug, thiserror::Error)]
pub enum CourseProgressError {
    #[error("sqlx: {0}")]
    Sqlx(#[from] sqlx::Error),
}

impl CourseProgressRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Mark a lesson complete (idempotent on `(enrollment_id, lesson_id)`)
    /// with optional time-spent + payload. Notes are left untouched here
    /// — use [`upsert_notes`] for that, so a user marking complete after
    /// taking notes doesn't accidentally wipe them.
    pub async fn mark_complete(
        &self,
        enrollment_id: EnrollmentId,
        lesson_id: &str,
        time_spent_seconds: i32,
        payload: Json,
    ) -> Result<(), CourseProgressError> {
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO course_progress (id, enrollment_id, lesson_id, time_spent_seconds, payload)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (enrollment_id, lesson_id) DO UPDATE
            SET time_spent_seconds = course_progress.time_spent_seconds + EXCLUDED.time_spent_seconds,
                payload = EXCLUDED.payload
            "#,
            id,
            enrollment_id.as_uuid(),
            lesson_id,
            time_spent_seconds,
            payload,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Upsert notes for a lesson. Creates the row at zero-progress if it
    /// doesn't exist (the user may take notes before clicking
    /// "complete"). The dashboard counts a lesson "complete" by the
    /// presence of the row PLUS `time_spent_seconds > 0` — notes alone
    /// do not flip completion.
    pub async fn upsert_notes(
        &self,
        enrollment_id: EnrollmentId,
        lesson_id: &str,
        notes: &str,
    ) -> Result<(), CourseProgressError> {
        let id = Uuid::now_v7();
        sqlx::query!(
            r#"
            INSERT INTO course_progress (id, enrollment_id, lesson_id, notes)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (enrollment_id, lesson_id) DO UPDATE
            SET notes = EXCLUDED.notes
            "#,
            id,
            enrollment_id.as_uuid(),
            lesson_id,
            notes,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_for_enrollment(
        &self,
        enrollment_id: EnrollmentId,
    ) -> Result<Vec<LessonProgress>, CourseProgressError> {
        let rows = sqlx::query!(
            r#"
            SELECT id, enrollment_id, lesson_id, completed_at, notes,
                   time_spent_seconds, payload
            FROM course_progress
            WHERE enrollment_id = $1
            ORDER BY completed_at ASC
            "#,
            enrollment_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|r| LessonProgress {
                id: r.id,
                enrollment_id: EnrollmentId::from_uuid(r.enrollment_id),
                lesson_id: r.lesson_id,
                completed_at: r.completed_at,
                notes: r.notes,
                time_spent_seconds: r.time_spent_seconds,
                payload: r.payload,
            })
            .collect())
    }

    pub async fn count_completed(
        &self,
        enrollment_id: EnrollmentId,
    ) -> Result<i64, CourseProgressError> {
        let row = sqlx::query!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM course_progress
            WHERE enrollment_id = $1 AND time_spent_seconds > 0
            "#,
            enrollment_id.as_uuid(),
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.count)
    }
}
