//! `GET /v1/admin/leads` — list newest leads (default 200).

use axum::extract::{Query, State};
use axum::Json;
use common::error::AppError;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct LeadRow {
    pub id: String,
    pub email: String,
    pub source: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    pub leads: Vec<LeadRow>,
}

const MAX_LIMIT: i64 = 1000;
const DEFAULT_LIMIT: i64 = 200;

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<ListResponse>, AppError> {
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let leads = state
        .leads
        .list(limit)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(ListResponse {
        leads: leads
            .into_iter()
            .map(|l| LeadRow {
                id: l.id.to_string(),
                email: l.email,
                source: l.source,
                created_at: l.created_at,
            })
            .collect(),
    }))
}
