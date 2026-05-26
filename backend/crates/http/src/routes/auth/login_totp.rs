//! `POST /v1/auth/login/totp` — stage-2 of two-step login.
//!
//! Accepts `{tx_id, code}` where:
//! - `tx_id` was minted by stage-1 `/v1/auth/login` and is HMAC-signed with
//!   `AUTH_COOKIE_KEY` (BACKEND_NOTES PR #5). 5-minute TTL.
//! - `code` is either a 6-digit TOTP or a 10-char backup code. The handler
//!   dispatches in `twofa::verify_code_any` (same module that powers
//!   `/v1/me/2fa/disable`'s code check).
//!
//! Per-IP login limiter is consulted here too so a successful stage-1 cannot
//! bypass it for unlimited stage-2 attempts.

use axum::extract::{ConnectInfo, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::auth::{pending_totp_token, session_cookie};
use common::error::AppError;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use validator::Validate;

use crate::routes::auth::signup::{format_cookie, UserPayload};
use crate::routes::me::twofa::verify_code_any;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginTotpRequest {
    #[validate(length(min = 1))]
    pub tx_id: String,
    /// 6-digit TOTP OR 10-char backup code.
    #[validate(length(min = 6, max = 10))]
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct LoginTotpResponse {
    pub user: UserPayload,
}

pub async fn handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<LoginTotpRequest>,
) -> Result<Response, AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("invalid totp fields".into()))?;

    // Stage-2 also burns one bucket entry — otherwise an attacker who got
    // past stage 1 could pound TOTP codes without throttling.
    state
        .limiter
        .check(crate::middleware::rate_limit::Bucket::Login, peer.ip())
        .map_err(|e| {
            tracing::warn!(peer = %peer.ip(), "login/totp attempts exhausted");
            AppError::RateLimited {
                retry_after_secs: e.retry_after_secs,
            }
        })?;

    let user_id = pending_totp_token::verify(&body.tx_id, &state.cookie_key).map_err(|e| {
        tracing::info!(error = ?e, "login/totp: tx_id invalid");
        AppError::Unauthorized
    })?;

    let user = state
        .users
        .find_by_id(user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Unauthorized)?;
    if user.totp_enabled_at.is_none() {
        // The tx_id was for a user that has since disabled 2FA. Reject —
        // the caller should restart at /v1/auth/login (which will then
        // succeed without the second step).
        return Err(AppError::Unauthorized);
    }

    let ok = verify_code_any(&state, user.id, &body.code).await?;
    if !ok {
        tracing::info!(user_id = %user.id, "login/totp: bad code");
        return Err(AppError::Unauthorized);
    }

    let issued = session_cookie::issue(&state.cookie_key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    state
        .sessions
        .create(
            issued.session_id,
            user.id,
            &issued.token_hash,
            None,
            Some(peer.ip()),
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let mut resp = (
        StatusCode::OK,
        Json(LoginTotpResponse {
            user: UserPayload {
                id: user.id.to_string(),
                email: user.email,
                name: user.name,
                role: user.role,
            },
        }),
    )
        .into_response();
    let cookie = format_cookie(&state, &issued.cookie_value)?;
    resp.headers_mut().insert(header::SET_COOKIE, cookie);
    Ok(resp)
}
