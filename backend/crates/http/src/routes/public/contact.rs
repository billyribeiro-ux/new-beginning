//! `POST /v1/public/contact`.
//!
//! Mirrors Phase-1 validator: `name` 1–80, `email` valid, `subject` 2–160,
//! `body` 10–4000, `website` honeypot.

use axum::extract::{ConnectInfo, State};
use axum::http::StatusCode;
use axum::Json;
use common::error::AppError;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use validator::Validate;

use crate::middleware::rate_limit::Bucket;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct ContactRequest {
    #[validate(length(min = 1, max = 80))]
    pub name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 2, max = 160))]
    pub subject: String,
    #[validate(length(min = 10, max = 4000))]
    pub body: String,
    #[serde(default)]
    pub website: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContactResponse {
    pub status: &'static str,
}

pub async fn submit(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<ContactRequest>,
) -> Result<(StatusCode, Json<ContactResponse>), AppError> {
    state
        .limiter
        .check(Bucket::Contact, peer.ip())
        .map_err(|e| AppError::RateLimited {
            retry_after_secs: e.retry_after_secs,
        })?;

    // Honeypot first.
    if let Some(w) = &body.website {
        if !w.trim().is_empty() {
            tracing::info!(peer = %peer.ip(), "honeypot tripped on /v1/public/contact");
            return Ok((StatusCode::CREATED, Json(ContactResponse { status: "ok" })));
        }
    }

    body.validate()
        .map_err(|_| AppError::Validation("name, email, subject, body required".into()))?;

    state
        .contact
        .create(
            body.name.trim(),
            body.email.trim(),
            body.subject.trim(),
            body.body.trim(),
            Some(peer.ip()),
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok((StatusCode::CREATED, Json(ContactResponse { status: "ok" })))
}
