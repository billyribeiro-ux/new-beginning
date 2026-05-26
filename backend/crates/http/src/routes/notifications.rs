//! `/v1/notifications/*` — in-app feed + preferences.
//!
//! BACKEND.md §11. Five endpoints:
//!
//! * `GET /v1/notifications?unread=true&limit=50` — feed (paginated;
//!   `limit` capped at 200).
//! * `PATCH /v1/notifications/{id}/read` — mark a single row read.
//!   Returns 204; idempotent (already-read is a no-op).
//! * `POST /v1/notifications/mark-all-read` — bulk mark; returns
//!   `{updated: n}`.
//! * `GET /v1/notifications/preferences` — current opt-in matrix.
//!   Lazy-materializes the row from defaults if it doesn't exist.
//! * `PATCH /v1/notifications/preferences` — partial update.
//!   Double-Option semantics on DND times allow explicit null.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use common::deserialize::double_option;
use common::error::AppError;
use common::ids::NotificationId;
use serde::{Deserialize, Serialize};
use storage::PreferencesPatch;
use time::{OffsetDateTime, Time};

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub unread: Option<bool>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct FeedRow {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub source: String,
    pub read: bool,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub notifications: Vec<FeedRow>,
    pub unread_count: i64,
}

/// `limit` cap. A higher value would let a single request stream the
/// entire feed; the BFF paginates above this with `before=<created_at>`
/// (a follow-up — today the limit-only model is fine).
const MAX_LIMIT: i64 = 200;
const DEFAULT_LIMIT: i64 = 50;

pub async fn list(
    State(state): State<AppState>,
    session: AuthSession,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let unread_only = q.unread.unwrap_or(false);
    let rows = state
        .notifications
        .list_for_user(session.user_id, unread_only, limit)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let unread_count = state
        .notifications
        .unread_count(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(ListResponse {
        notifications: rows
            .into_iter()
            .map(|n| FeedRow {
                id: n.id.to_string(),
                kind: n.kind,
                title: n.title,
                body: n.body,
                source: n.source,
                read: n.read_at.is_some(),
                created_at: n.created_at,
            })
            .collect(),
        unread_count,
    }))
}

pub async fn mark_read(
    State(state): State<AppState>,
    session: AuthSession,
    Path(id): Path<NotificationId>,
) -> Result<StatusCode, AppError> {
    // We DON'T 404 on a no-op; a row that's already read returns 204
    // (idempotent). A row owned by another user returns 204 too — the
    // user_id predicate is the entitlement and silently no-ops on the
    // wrong owner. The alternative (404 on wrong-owner) leaks the
    // existence of the id to a probing attacker.
    state
        .notifications
        .mark_read(session.user_id, id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct MarkAllReadResponse {
    pub updated: u64,
}

pub async fn mark_all_read(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<MarkAllReadResponse>, AppError> {
    let updated = state
        .notifications
        .mark_all_read(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(MarkAllReadResponse { updated }))
}

#[derive(Debug, Serialize)]
pub struct PreferencesResponse {
    pub product_updates_email: bool,
    pub product_updates_inapp: bool,
    pub billing_email: bool,
    pub billing_inapp: bool,
    pub course_progress_email: bool,
    pub course_progress_inapp: bool,
    pub market_alerts_email: bool,
    pub market_alerts_inapp: bool,
    pub marketing_email: bool,
    pub marketing_inapp: bool,
    pub dnd_enabled: bool,
    pub dnd_start: Option<Time>,
    pub dnd_end: Option<Time>,
    pub timezone: String,
}

pub async fn get_prefs(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<PreferencesResponse>, AppError> {
    let p = state
        .notification_prefs
        .get_or_default(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(to_response(p)))
}

#[derive(Debug, Deserialize, Default)]
pub struct PatchRequest {
    #[serde(default)]
    pub product_updates_email: Option<bool>,
    #[serde(default)]
    pub product_updates_inapp: Option<bool>,
    #[serde(default)]
    pub billing_email: Option<bool>,
    #[serde(default)]
    pub billing_inapp: Option<bool>,
    #[serde(default)]
    pub course_progress_email: Option<bool>,
    #[serde(default)]
    pub course_progress_inapp: Option<bool>,
    #[serde(default)]
    pub market_alerts_email: Option<bool>,
    #[serde(default)]
    pub market_alerts_inapp: Option<bool>,
    #[serde(default)]
    pub marketing_email: Option<bool>,
    #[serde(default)]
    pub marketing_inapp: Option<bool>,
    #[serde(default)]
    pub dnd_enabled: Option<bool>,
    #[serde(default, deserialize_with = "double_option")]
    pub dnd_start: Option<Option<Time>>,
    #[serde(default, deserialize_with = "double_option")]
    pub dnd_end: Option<Option<Time>>,
    #[serde(default)]
    pub timezone: Option<String>,
}

pub async fn patch_prefs(
    State(state): State<AppState>,
    session: AuthSession,
    Json(req): Json<PatchRequest>,
) -> Result<Json<PreferencesResponse>, AppError> {
    // If dnd_enabled is being set to TRUE, dnd_start AND dnd_end MUST
    // either be set in this request OR already present (CHECK constraint
    // on the table). Surfacing as 422 vs. a DB-level error gives a
    // cleaner UX.
    if req.dnd_enabled == Some(true) {
        let current = state
            .notification_prefs
            .get_or_default(session.user_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let final_start = req.dnd_start.unwrap_or(current.dnd_start);
        let final_end = req.dnd_end.unwrap_or(current.dnd_end);
        if final_start.is_none() || final_end.is_none() {
            return Err(AppError::Validation(
                "dnd_enabled=true requires dnd_start and dnd_end".into(),
            ));
        }
    }
    let patch = PreferencesPatch {
        product_updates_email: req.product_updates_email,
        product_updates_inapp: req.product_updates_inapp,
        billing_email: req.billing_email,
        billing_inapp: req.billing_inapp,
        course_progress_email: req.course_progress_email,
        course_progress_inapp: req.course_progress_inapp,
        market_alerts_email: req.market_alerts_email,
        market_alerts_inapp: req.market_alerts_inapp,
        marketing_email: req.marketing_email,
        marketing_inapp: req.marketing_inapp,
        dnd_enabled: req.dnd_enabled,
        dnd_start: req.dnd_start,
        dnd_end: req.dnd_end,
        timezone: req.timezone,
    };
    let p = state
        .notification_prefs
        .patch(session.user_id, patch)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(to_response(p)))
}

fn to_response(p: storage::NotificationPreferences) -> PreferencesResponse {
    PreferencesResponse {
        product_updates_email: p.product_updates_email,
        product_updates_inapp: p.product_updates_inapp,
        billing_email: p.billing_email,
        billing_inapp: p.billing_inapp,
        course_progress_email: p.course_progress_email,
        course_progress_inapp: p.course_progress_inapp,
        market_alerts_email: p.market_alerts_email,
        market_alerts_inapp: p.market_alerts_inapp,
        marketing_email: p.marketing_email,
        marketing_inapp: p.marketing_inapp,
        dnd_enabled: p.dnd_enabled,
        dnd_start: p.dnd_start,
        dnd_end: p.dnd_end,
        timezone: p.timezone,
    }
}
