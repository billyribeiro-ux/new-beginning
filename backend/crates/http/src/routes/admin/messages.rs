//! `GET /v1/admin/messages` + `PATCH /v1/admin/messages/{id}`.
//!
//! Closed-set status updates {`new`, `read`, `archived`, `spam`}. PATCH
//! audits the actor + transition.

use axum::extract::{Path, Query, State};
use axum::Json;
use common::error::AppError;
use common::ids::ContactMessageId;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct MessageRow {
    pub id: String,
    pub name: String,
    pub email: String,
    pub subject: String,
    pub body: String,
    pub status: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub messages: Vec<MessageRow>,
}

const MAX_LIMIT: i64 = 500;
const DEFAULT_LIMIT: i64 = 200;
const ALLOWED_STATUSES: &[&str] = &["new", "read", "archived", "spam"];

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    if let Some(s) = q.status.as_deref() {
        if !ALLOWED_STATUSES.contains(&s) {
            return Err(AppError::Validation(format!(
                "status must be one of {ALLOWED_STATUSES:?}"
            )));
        }
    }
    let rows = state
        .contact
        .list(q.status.as_deref(), limit)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(ListResponse {
        messages: rows
            .into_iter()
            .map(|m| MessageRow {
                id: m.id.to_string(),
                name: m.name,
                email: m.email,
                subject: m.subject,
                body: m.body,
                status: m.status,
                created_at: m.created_at,
            })
            .collect(),
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct PatchRequest {
    #[validate(length(min = 1, max = 16))]
    pub status: String,
}

pub async fn patch(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<ContactMessageId>,
    Json(req): Json<PatchRequest>,
) -> Result<axum::http::StatusCode, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    if !ALLOWED_STATUSES.contains(&req.status.as_str()) {
        return Err(AppError::Validation(format!(
            "status must be one of {ALLOWED_STATUSES:?}"
        )));
    }
    // Audit + status update in one tx (BACKEND.md §22 rule 8).
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let n = sqlx::query!(
        "UPDATE contact_messages SET status = $2 WHERE id = $1",
        id.as_uuid(),
        &req.status,
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    .rows_affected();
    if n == 0 {
        // Rollback (no row touched) — return 404. Audit row would record
        // a no-op; we skip it.
        let _ = tx.rollback().await;
        return Err(AppError::NotFound);
    }
    state
        .audit
        .record_in_tx(
            &mut tx,
            Some(session.user_id),
            "admin.message.status_changed",
            "contact_messages",
            &id.to_string(),
            serde_json::json!({ "status": &req.status }),
            None,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}
