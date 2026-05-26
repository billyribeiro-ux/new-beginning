//! `POST /v1/me/change-password` — rotate the user's password.
//!
//! Requires the current password (BACKEND.md §19) and revokes every OTHER
//! session (BACKEND.md §7.5 spirit — rotating a credential should expire
//! other agents holding it). The caller's own session stays alive so they
//! aren't logged out by their own action.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use common::auth::password::{hash_password, verify_password};
use common::error::AppError;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
    #[validate(length(min = 1))]
    pub current_password: String,
    #[validate(length(min = 8, max = 128))]
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    /// Number of other sessions that were revoked (handy for the UI to
    /// surface "you're signed out everywhere else").
    pub other_sessions_revoked: u64,
}

pub async fn handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<(StatusCode, Json<ChangePasswordResponse>), AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("password fields invalid".into()))?;
    enforce_strength(&body.new_password)?;

    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Unauthorized)?;

    let Some(stored) = user.password_hash.clone() else {
        // No password set → reject. (Federated logins land in a later PR;
        // this branch becomes "use the SSO flow" then.)
        return Err(AppError::Unauthorized);
    };

    // Verify-current FIRST, under the hash-concurrency permit.
    let permit = state
        .hash_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
    let ok = verify_password(
        SecretString::from(body.current_password.clone()),
        stored,
        state.password_pepper.clone(),
    )
    .await;
    drop(permit);
    let ok = ok.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !ok {
        return Err(AppError::Unauthorized);
    }

    // Hash the new password under the same cap.
    let permit = state
        .hash_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
    let new_hash = hash_password(
        SecretString::from(body.new_password.clone()),
        state.password_pepper.clone(),
    )
    .await;
    drop(permit);
    let new_hash = new_hash.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    state
        .users
        .update_password(session.user_id, &new_hash)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let revoked = state
        .sessions
        .revoke_all_except(session.user_id, session.session_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    tracing::info!(
        user_id = %session.user_id,
        revoked,
        "password changed; revoked other sessions"
    );

    Ok((
        StatusCode::OK,
        Json(ChangePasswordResponse {
            other_sessions_revoked: revoked,
        }),
    ))
}

fn enforce_strength(pw: &str) -> Result<(), AppError> {
    let has_upper = pw.chars().any(|c| c.is_ascii_uppercase());
    let has_lower = pw.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = pw.chars().any(|c| c.is_ascii_digit());
    if has_upper && has_lower && has_digit {
        Ok(())
    } else {
        Err(AppError::Validation(
            "password must include an uppercase letter, a lowercase letter, and a digit".into(),
        ))
    }
}
