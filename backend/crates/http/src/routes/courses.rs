//! `/v1/courses/*` — enrollment list + course player state + progress.
//!
//! BACKEND.md §11 (endpoint surface). Four endpoints:
//!
//! * `GET /v1/courses` — every active enrollment + product metadata
//!   (dashboard "My Courses" widget).
//! * `GET /v1/courses/{slug}` — full player state for one course:
//!   product info, manifest (modules + lessons), per-lesson completion,
//!   last-lesson cursor.
//! * `POST /v1/courses/{slug}/progress` — mark a lesson complete; also
//!   recomputes `enrollments.progress_pct`.
//! * `PUT /v1/courses/{slug}/lessons/{lesson_id}/notes` — upsert notes
//!   without flipping completion.
//!
//! The course "manifest" (modules + lessons) is currently read from the
//! product's `specs_json` blob, shape:
//!   `{"modules":[{"id":"m1","title":"…","lessons":[{"id":"m1-l1","title":"…"}]}]}`
//! PR #14 (admin) ships the CRUD that lets an admin edit `specs_json`.

use axum::extract::{Path, State};
use axum::Json;
use common::error::AppError;
use common::ids::EnrollmentId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct CourseRow {
    pub product_id: String,
    pub product_slug: String,
    pub product_name: String,
    pub progress_pct: i32,
    pub last_lesson_id: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub courses: Vec<CourseRow>,
}

pub async fn list(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<ListResponse>, AppError> {
    // Fetch active enrollments + their progress rows. The repo's
    // list_for_user already filters to active = TRUE.
    let enrollments = state
        .enrollments
        .list_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let mut rows = Vec::with_capacity(enrollments.len());
    for enr in enrollments {
        let product = state
            .products
            .find_by_id(enr.product_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let Some(product) = product else {
            continue;
        };
        // Pull progress_pct + last_lesson_id from the enrollment row.
        // The repo's list_for_user model doesn't include those columns;
        // re-query with the explicit lookup.
        let with_progress = state
            .enrollments
            .find_for_user_and_product(session.user_id, enr.product_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        // The combined helper returned the active row above; reusing it
        // here for the progress fields keeps logic in one place. Pull
        // the full row through a SQL query.
        let (progress_pct, last_lesson_id) = match with_progress {
            Some(_) => fetch_progress_fields(&state, enr.id).await?,
            None => (0, None),
        };

        rows.push(CourseRow {
            product_id: enr.product_id.to_string(),
            product_slug: product.slug,
            product_name: product.name,
            progress_pct,
            last_lesson_id,
            started_at: enr.started_at,
        });
    }
    Ok(Json(ListResponse { courses: rows }))
}

async fn fetch_progress_fields(
    state: &AppState,
    enrollment_id: EnrollmentId,
) -> Result<(i32, Option<String>), AppError> {
    let row = sqlx::query!(
        r#"
        SELECT progress_pct, last_lesson_id
        FROM enrollments
        WHERE id = $1
        "#,
        enrollment_id.as_uuid(),
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok((row.progress_pct, row.last_lesson_id))
}

#[derive(Debug, Serialize)]
pub struct LessonState {
    pub id: String,
    pub title: String,
    pub completed: bool,
    pub time_spent_seconds: i32,
    pub notes: String,
}

#[derive(Debug, Serialize)]
pub struct ModuleState {
    pub id: String,
    pub title: String,
    pub lessons: Vec<LessonState>,
}

#[derive(Debug, Serialize)]
pub struct PlayerStateResponse {
    pub product_id: String,
    pub product_slug: String,
    pub product_name: String,
    pub progress_pct: i32,
    pub last_lesson_id: Option<String>,
    pub modules: Vec<ModuleState>,
}

#[derive(Deserialize)]
struct CourseManifest {
    modules: Vec<ManifestModule>,
}
#[derive(Deserialize)]
struct ManifestModule {
    id: String,
    title: String,
    lessons: Vec<ManifestLesson>,
}
#[derive(Deserialize)]
struct ManifestLesson {
    id: String,
    title: String,
}

pub async fn player_state(
    State(state): State<AppState>,
    session: AuthSession,
    Path(slug): Path<String>,
) -> Result<Json<PlayerStateResponse>, AppError> {
    let product = state
        .products
        .find_by_slug(&slug)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;
    let enrollment = state
        .enrollments
        .find_for_user_and_product(session.user_id, product.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Forbidden)?;

    let manifest = parse_manifest(&product.specs_json);

    let progress = state
        .course_progress
        .list_for_enrollment(enrollment.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let (progress_pct, last_lesson_id) = fetch_progress_fields(&state, enrollment.id).await?;

    let modules = manifest
        .modules
        .into_iter()
        .map(|m| ModuleState {
            id: m.id,
            title: m.title,
            lessons: m
                .lessons
                .into_iter()
                .map(|l| {
                    let p = progress.iter().find(|p| p.lesson_id == l.id);
                    LessonState {
                        id: l.id,
                        title: l.title,
                        completed: p.map(|p| p.time_spent_seconds > 0).unwrap_or(false),
                        time_spent_seconds: p.map(|p| p.time_spent_seconds).unwrap_or(0),
                        notes: p.map(|p| p.notes.clone()).unwrap_or_default(),
                    }
                })
                .collect(),
        })
        .collect();

    Ok(Json(PlayerStateResponse {
        product_id: product.id.to_string(),
        product_slug: product.slug,
        product_name: product.name,
        progress_pct,
        last_lesson_id,
        modules,
    }))
}

fn parse_manifest(specs: &serde_json::Value) -> CourseManifest {
    serde_json::from_value::<CourseManifest>(specs.clone())
        .unwrap_or(CourseManifest { modules: vec![] })
}

#[derive(Debug, Deserialize, Validate)]
pub struct ProgressRequest {
    #[validate(length(min = 1, max = 128))]
    pub lesson_id: String,
    #[validate(range(min = 0, max = 86_400))]
    pub time_spent_seconds: i32,
    #[serde(default)]
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ProgressResponse {
    pub progress_pct: i32,
}

pub async fn post_progress(
    State(state): State<AppState>,
    session: AuthSession,
    Path(slug): Path<String>,
    Json(req): Json<ProgressRequest>,
) -> Result<Json<ProgressResponse>, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let product = state
        .products
        .find_by_slug(&slug)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;
    let enrollment = state
        .enrollments
        .find_for_user_and_product(session.user_id, product.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Forbidden)?;

    // Validate the lesson_id against the product's manifest. A request
    // for an unknown lesson is a 400 — refusing to silently store
    // arbitrary text keeps the table on a closed vocabulary.
    let manifest = parse_manifest(&product.specs_json);
    let total_lessons: usize = manifest.modules.iter().map(|m| m.lessons.len()).sum();
    let lesson_exists = manifest
        .modules
        .iter()
        .any(|m| m.lessons.iter().any(|l| l.id == req.lesson_id));
    if !lesson_exists {
        return Err(AppError::Validation(format!(
            "lesson_id {:?} not in course manifest",
            req.lesson_id
        )));
    }

    state
        .course_progress
        .mark_complete(
            enrollment.id,
            &req.lesson_id,
            req.time_spent_seconds,
            req.payload,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let completed = state
        .course_progress
        .count_completed(enrollment.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let progress_pct = if total_lessons == 0 {
        0
    } else {
        ((completed as i64 * 100) / total_lessons as i64).clamp(0, 100) as i32
    };
    state
        .enrollments
        .update_progress(enrollment.id, progress_pct, Some(&req.lesson_id))
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(ProgressResponse { progress_pct }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct NotesRequest {
    #[validate(length(max = 16_384))]
    pub notes: String,
}

pub async fn put_notes(
    State(state): State<AppState>,
    session: AuthSession,
    Path((slug, lesson_id)): Path<(String, String)>,
    Json(req): Json<NotesRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    let product = state
        .products
        .find_by_slug(&slug)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;
    let enrollment = state
        .enrollments
        .find_for_user_and_product(session.user_id, product.id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Forbidden)?;

    let manifest = parse_manifest(&product.specs_json);
    let lesson_exists = manifest
        .modules
        .iter()
        .any(|m| m.lessons.iter().any(|l| l.id == lesson_id));
    if !lesson_exists {
        return Err(AppError::NotFound);
    }

    state
        .course_progress
        .upsert_notes(enrollment.id, &lesson_id, &req.notes)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
