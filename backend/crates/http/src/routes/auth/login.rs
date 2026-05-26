//! `POST /v1/auth/login`.
//!
//! BACKEND.md §19: stage-1 of two-step login.
//!
//! - 2FA off: returns 200 + sets cookie. Done.
//! - 2FA on : returns 200 + `{step: "totp", tx_id}` — NO cookie set; the
//!   caller follows up with `POST /v1/auth/login/totp { tx_id, code }`
//!   (see `login_totp.rs`).
//!
//! Lockout: BACKEND.md §16.3 — `LOGIN_BUCKET_PER_MINUTE` attempts from one IP
//! → 429 with `Retry-After`. The limiter runs BEFORE the password verify so
//! an attacker cannot burn Argon2 cycles.

use axum::extract::{ConnectInfo, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use common::auth::{password::verify_password, pending_totp_token, session_cookie};
use common::error::AppError;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use validator::Validate;

use crate::routes::auth::signup::{format_cookie, UserPayload};
use crate::state::AppState;

/// `tx_id` TTL between stage 1 and stage 2 (BACKEND_NOTES PR #5).
const PENDING_TOTP_TTL: Duration = Duration::from_secs(5 * 60);

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1))]
    pub password: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "step")]
pub enum LoginResponse {
    /// `"step": "done"` — cookie set, user fully authenticated.
    #[serde(rename = "done")]
    Done { user: UserPayload },
    /// `"step": "totp"` — caller must POST the code to /v1/auth/login/totp
    /// with `tx_id`.
    #[serde(rename = "totp")]
    Totp { tx_id: String },
}

pub async fn handler(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    Json(body): Json<LoginRequest>,
) -> Result<Response, AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("invalid login fields".into()))?;

    state
        .limiter
        .check(crate::middleware::rate_limit::Bucket::Login, peer.ip())
        .map_err(|e| {
            tracing::warn!(peer = %peer.ip(), "login attempts exhausted");
            AppError::RateLimited {
                retry_after_secs: e.retry_after_secs,
            }
        })?;

    let user = state
        .users
        .find_by_email(&body.email)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let Some(user) = user else {
        tracing::info!(email = %body.email, "login: unknown email");
        return Err(AppError::Unauthorized);
    };
    let Some(stored) = user.password_hash.clone() else {
        tracing::warn!(user_id = %user.id, "login: user has no password_hash");
        return Err(AppError::Unauthorized);
    };

    let permit = state
        .hash_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
    let ok = verify_password(
        SecretString::from(body.password.clone()),
        stored,
        state.password_pepper.clone(),
    )
    .await;
    drop(permit);
    let ok = ok.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !ok {
        tracing::info!(user_id = %user.id, "login: bad password");
        return Err(AppError::Unauthorized);
    }

    // 2FA branch: do NOT set the cookie; mint a pending_totp token.
    if user.totp_enabled_at.is_some() {
        let tx_id = pending_totp_token::issue(user.id, PENDING_TOTP_TTL, &state.cookie_key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        tracing::info!(user_id = %user.id, "login: password ok, 2fa required");
        return Ok((StatusCode::OK, Json(LoginResponse::Totp { tx_id })).into_response());
    }

    // 2FA off: mint the session cookie immediately.
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
        Json(LoginResponse::Done {
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
