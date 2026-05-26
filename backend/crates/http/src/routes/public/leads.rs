//! `POST /v1/public/leads` — lead capture.
//!
//! Mirrors STACK.md / Phase-1 validator: `email` (valid), `source` (≤ 64
//! chars, defaults to `"free-guide"`), `website` honeypot (any non-empty
//! value → silent 201 + no insert).

use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::Json;
use axum_extra::headers::UserAgent;
use axum_extra::TypedHeader;
use common::error::AppError;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use validator::Validate;

use crate::middleware::rate_limit::Bucket;
use crate::state::AppState;

const DEFAULT_SOURCE: &str = "free-guide";

#[derive(Debug, Deserialize, Validate)]
pub struct LeadRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 64))]
    pub source: Option<String>,
    /// Honeypot: forms render this as a hidden input. Real users leave it
    /// blank; bots auto-fill every field.
    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LeadResponse {
    pub status: &'static str,
}

pub async fn capture(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    user_agent: Option<TypedHeader<UserAgent>>,
    Json(body): Json<LeadRequest>,
) -> Result<(StatusCode, Json<LeadResponse>), AppError> {
    state
        .limiter
        .check(Bucket::LeadCapture, peer.ip())
        .map_err(|e| AppError::RateLimited {
            retry_after_secs: e.retry_after_secs,
        })?;

    // Honeypot first — silent 201 keeps bots in the dark.
    if let Some(w) = &body.website {
        if !w.trim().is_empty() {
            tracing::info!(peer = %peer.ip(), "honeypot tripped on /v1/public/leads");
            return Ok((StatusCode::CREATED, Json(LeadResponse { status: "ok" })));
        }
    }

    body.validate()
        .map_err(|_| AppError::Validation("email is required".into()))?;
    let source = body
        .source
        .as_deref()
        .unwrap_or(DEFAULT_SOURCE)
        .trim()
        .to_string();
    let ua = user_agent.as_ref().map(|TypedHeader(u)| u.as_str());

    state
        .leads
        .create(&body.email, &source, Some(peer.ip()), ua)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok((StatusCode::CREATED, Json(LeadResponse { status: "ok" })))
}
