//! `GET /v1/public/plans`.

use axum::extract::{ConnectInfo, State};
use axum::Json;
use common::error::AppError;
use serde::Serialize;
use std::net::SocketAddr;

use crate::middleware::rate_limit::Bucket;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub plans: Vec<PlanPayload>,
}

#[derive(Debug, Serialize)]
pub struct PlanPayload {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub cadence: String,
    pub price_cents: i64,
    pub monthly_equivalent_cents: i64,
    pub savings_pct: i32,
    pub tagline: String,
    pub highlights: Vec<String>,
    pub featured: bool,
    pub badge: Option<String>,
}

impl PlanPayload {
    fn from_repo(p: storage::SubscriptionPlan) -> Self {
        Self {
            id: p.id.to_string(),
            slug: p.slug,
            name: p.name,
            cadence: p.cadence,
            price_cents: p.price_cents,
            monthly_equivalent_cents: p.monthly_equivalent_cents,
            savings_pct: p.savings_pct,
            tagline: p.tagline,
            highlights: p.highlights,
            featured: p.featured,
            badge: p.badge,
        }
    }
}

pub async fn list(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
) -> Result<Json<ListResponse>, AppError> {
    state
        .limiter
        .check(Bucket::PublicRead, peer.ip())
        .map_err(|e| AppError::RateLimited {
            retry_after_secs: e.retry_after_secs,
        })?;

    let rows = state
        .plans
        .list_all()
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(ListResponse {
        plans: rows.into_iter().map(PlanPayload::from_repo).collect(),
    }))
}
