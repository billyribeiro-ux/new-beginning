//! `GET /v1/admin/stats` — KPIs for the admin dashboard.
//!
//! BACKEND.md §11. Returns:
//!   * `revenue_30d_cents` — sum of `total_cents` over paid orders in
//!     the last 30 days.
//!   * `orders_30d` — count of those orders.
//!   * `active_subscriptions` — current count of subs in
//!     {trialing, active, past_due}. This is a count proxy for MRR until
//!     PR #14 admin plan CRUD populates `subscription_plans.price_cents`
//!     for real plan-level multiplication.
//!   * `signups_30d` — `users` created since cutoff.
//!   * `leads_30d` — `leads` created since cutoff.

use axum::extract::State;
use axum::Json;
use common::error::AppError;
use serde::Serialize;
use time::{Duration, OffsetDateTime};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub revenue_30d_cents: i64,
    pub orders_30d: i64,
    pub active_subscriptions: i64,
    pub signups_30d: i64,
    pub leads_30d: i64,
}

pub async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, AppError> {
    let since = OffsetDateTime::now_utc() - Duration::days(30);

    let (orders_30d, revenue_30d_cents) = state
        .orders
        .revenue_since(since)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let active_subscriptions = state
        .subscriptions
        .count_active()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let signups_30d = state
        .users
        .count_since(since)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let leads_30d = state
        .leads
        .count_since(since)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(StatsResponse {
        revenue_30d_cents,
        orders_30d,
        active_subscriptions,
        signups_30d,
        leads_30d,
    }))
}
