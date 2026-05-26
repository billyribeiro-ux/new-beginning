//! `GET /v1/subscription` — caller's active subscription mirror.
//!
//! BACKEND.md §19. Read-only in PR #8; mutation endpoints
//! (`change-plan` / `pause` / `resume` / `cancel`) land in PR #9 via the
//! Stripe Customer Portal.

use axum::extract::State;
use axum::Json;
use common::error::AppError;
use serde::Serialize;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SubscriptionPayload {
    pub id: String,
    pub plan_id: String,
    pub stripe_subscription_id: String,
    pub status: String,
    pub cancel_at_period_end: bool,
    pub current_period_start: String,
    pub current_period_end: String,
}

#[derive(Debug, Serialize)]
pub struct SubscriptionResponse {
    /// `null` when the caller has no active subscription.
    pub subscription: Option<SubscriptionPayload>,
}

pub async fn get(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<SubscriptionResponse>, AppError> {
    let row = state
        .subscriptions
        .find_active_for_user(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(SubscriptionResponse {
        subscription: row.map(|s| SubscriptionPayload {
            id: s.id.to_string(),
            plan_id: s.plan_id.to_string(),
            stripe_subscription_id: s.stripe_subscription_id,
            status: s.status,
            cancel_at_period_end: s.cancel_at_period_end,
            current_period_start: s
                .current_period_start
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
            current_period_end: s
                .current_period_end
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        }),
    }))
}
