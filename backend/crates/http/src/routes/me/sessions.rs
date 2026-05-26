//! `GET /v1/me/sessions`, `DELETE /v1/me/sessions/{id}`,
//! `POST /v1/me/sessions/revoke-others`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use common::error::AppError;
use common::ids::SessionId;
use serde::Serialize;
use uuid::Uuid;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SessionItem {
    pub id: String,
    /// True for the session that issued THIS request — so the UI can
    /// label "this device" without inferring from user-agent.
    pub current: bool,
    pub user_agent: Option<String>,
    pub ip: Option<String>,
    pub last_seen_at: String,
    pub expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub sessions: Vec<SessionItem>,
}

pub async fn list(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<ListResponse>, AppError> {
    let rows = state
        .sessions
        .list_active(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let items = rows
        .into_iter()
        .map(|r| SessionItem {
            id: r.id.to_string(),
            current: r.id == session.session_id,
            user_agent: r.user_agent,
            ip: r.ip.map(|i| i.to_string()),
            last_seen_at: r
                .last_seen_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            expires_at: r
                .expires_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        })
        .collect();
    Ok(Json(ListResponse { sessions: items }))
}

/// Revoke a single session. Returns 204 whether or not the row was found,
/// matching `logout` semantics — the property the caller wants is "this
/// session is no longer valid", which is true in both cases.
pub async fn revoke(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let target = SessionId::from_uuid(id);

    // Defense in depth: a user can only revoke their own sessions. We trust
    // the DB foreign key to user_id but enforce here to surface a clean 404
    // instead of a silent no-op.
    let own = state
        .sessions
        .list_active(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !own.iter().any(|s| s.id == target) {
        // The target either doesn't exist, belongs to a different user, or
        // was already revoked. All three look the same to the caller.
        return Ok(StatusCode::NO_CONTENT);
    }

    state
        .sessions
        .revoke(target)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    tracing::info!(
        user_id = %session.user_id,
        session_id = %target,
        "session revoked by user",
    );

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct RevokeOthersResponse {
    pub revoked: u64,
}

pub async fn revoke_others(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<RevokeOthersResponse>, AppError> {
    let revoked = state
        .sessions
        .revoke_all_except(session.user_id, session.session_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(RevokeOthersResponse { revoked }))
}
