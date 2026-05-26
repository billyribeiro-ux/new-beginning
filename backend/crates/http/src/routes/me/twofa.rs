//! `/v1/me/2fa/*` — enable / confirm / disable / regenerate-backup-codes.
//!
//! BACKEND.md §7.3 + §19. The flow:
//!
//! 1. `POST /enable` — server mints a fresh secret, encrypts it under
//!    `AUTH_TOTP_KEY`, stages it (resets `totp_enabled_at` to NULL), returns
//!    the otpauth URI + QR data URL so the user scans it.
//! 2. User scans, types the current 6-digit code.
//! 3. `POST /confirm` — server verifies code against the staged secret, sets
//!    `totp_enabled_at`, issues + hashes 10 backup codes, returns plaintext
//!    codes EXACTLY ONCE.
//! 4. Subsequent `POST /v1/auth/login` returns `{step:"totp", tx_id}`; the
//!    caller posts the code to `/v1/auth/login/totp`.
//!
//! Disable requires both the current password AND a current TOTP code so a
//! shoulder-surfed password can't drop 2FA.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use common::auth::backup_codes;
use common::auth::password::verify_password;
use common::auth::totp;
use common::auth::totp_secret_at_rest as totp_at_rest;
use common::error::AppError;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

const TOTP_ISSUER: &str = "TradeFlex";

#[derive(Debug, Serialize)]
pub struct EnableResponse {
    /// `otpauth://totp/...` — scannable URI for the authenticator app.
    pub otpauth_uri: String,
    /// `data:image/png;base64,...` — the same URI rendered as a QR PNG.
    pub qr_data_url: String,
}

/// `POST /v1/me/2fa/enable` — stage a fresh secret. Idempotent — calling it
/// again overwrites the staged secret and resets `totp_enabled_at`.
pub async fn enable(
    State(state): State<AppState>,
    session: AuthSession,
) -> Result<Json<EnableResponse>, AppError> {
    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    let secret = totp_at_rest::random_secret();
    let encrypted = totp_at_rest::encrypt(&secret, &state.totp_key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    state
        .users
        .set_totp_secret(session.user_id, &encrypted)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let totp = totp::build(&secret, TOTP_ISSUER, &user.email)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(Json(EnableResponse {
        otpauth_uri: totp::otpauth_uri(&totp),
        qr_data_url: totp::qr_data_url(&totp)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ConfirmRequest {
    #[validate(length(min = 6, max = 6))]
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct ConfirmResponse {
    /// 10 plaintext backup codes. **Surfaced exactly once.** The caller MUST
    /// show them to the user immediately; we keep only the hashes.
    pub backup_codes: Vec<String>,
}

/// `POST /v1/me/2fa/confirm` — verify first code, set `totp_enabled_at`,
/// generate + hash + persist backup codes, return plaintext codes.
pub async fn confirm(
    State(state): State<AppState>,
    session: AuthSession,
    Json(body): Json<ConfirmRequest>,
) -> Result<Json<ConfirmResponse>, AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("code must be 6 digits".into()))?;

    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;

    let blob = state
        .users
        .get_totp_secret(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Conflict("2FA setup not started".into()))?;
    let secret = totp_at_rest::decrypt(&blob, &state.totp_key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let totp_inst = totp::build(&secret, TOTP_ISSUER, &user.email)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    let ok =
        totp::verify(&totp_inst, &body.code).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    if !ok {
        return Err(AppError::Unauthorized);
    }

    // Generate + hash 10 backup codes. Each hash takes ~300ms with Argon2id
    // at m=64 MiB so this loop runs ~3s wall-clock under the semaphore.
    // Acceptable for a one-off user-initiated operation (BACKEND_NOTES PR
    // #5 decision).
    let plaintexts = backup_codes::generate_plaintext();
    let mut hashes: Vec<Vec<u8>> = Vec::with_capacity(plaintexts.len());
    for pt in &plaintexts {
        let permit = state
            .hash_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
        let h = backup_codes::hash_one(pt.clone(), state.password_pepper.clone()).await;
        drop(permit);
        hashes.push(h.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?);
    }

    state
        .users
        .enable_totp(session.user_id, &hashes)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(ConfirmResponse {
        backup_codes: plaintexts
            .into_iter()
            .map(|s| s.expose_secret().to_string())
            .collect(),
    }))
}

#[derive(Debug, Deserialize, Validate)]
pub struct DisableRequest {
    #[validate(length(min = 1))]
    pub current_password: String,
    /// Either a 6-digit TOTP code OR a 10-char backup code. Length-discriminated.
    #[validate(length(min = 6, max = 10))]
    pub code: String,
}

/// `POST /v1/me/2fa/disable` — wipe all 2FA state. Requires both current
/// password AND current code so a shoulder-surfed password cannot drop 2FA.
pub async fn disable(
    State(state): State<AppState>,
    session: AuthSession,
    Json(body): Json<DisableRequest>,
) -> Result<StatusCode, AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("invalid disable fields".into()))?;

    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Unauthorized)?;
    let Some(stored) = user.password_hash.clone() else {
        return Err(AppError::Unauthorized);
    };
    if user.totp_enabled_at.is_none() {
        return Err(AppError::Conflict("2FA is not enabled".into()));
    }

    // Verify password.
    let permit = state
        .hash_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
    let pw_ok = verify_password(
        SecretString::from(body.current_password.clone()),
        stored,
        state.password_pepper.clone(),
    )
    .await;
    drop(permit);
    if !pw_ok.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))? {
        return Err(AppError::Unauthorized);
    }

    // Verify code (TOTP or backup-code path).
    let code_ok = verify_code_any(&state, session.user_id, &body.code).await?;
    if !code_ok {
        return Err(AppError::Unauthorized);
    }

    state
        .users
        .disable_totp(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegenerateRequest {
    #[validate(length(min = 1))]
    pub current_password: String,
}

#[derive(Debug, Serialize)]
pub struct RegenerateResponse {
    pub backup_codes: Vec<String>,
}

/// `POST /v1/me/2fa/backup-codes/regenerate` — replace all 10 codes.
/// Invalidates the old set. Requires the current password (BACKEND_NOTES PR
/// #5).
pub async fn regenerate(
    State(state): State<AppState>,
    session: AuthSession,
    Json(body): Json<RegenerateRequest>,
) -> Result<Json<RegenerateResponse>, AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("password required".into()))?;
    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::Unauthorized)?;
    if user.totp_enabled_at.is_none() {
        return Err(AppError::Conflict("2FA is not enabled".into()));
    }
    let Some(stored) = user.password_hash.clone() else {
        return Err(AppError::Unauthorized);
    };

    let permit = state
        .hash_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
    let pw_ok = verify_password(
        SecretString::from(body.current_password.clone()),
        stored,
        state.password_pepper.clone(),
    )
    .await;
    drop(permit);
    if !pw_ok.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))? {
        return Err(AppError::Unauthorized);
    }

    let plaintexts = backup_codes::generate_plaintext();
    let mut hashes: Vec<Vec<u8>> = Vec::with_capacity(plaintexts.len());
    for pt in &plaintexts {
        let permit = state
            .hash_semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
        let h = backup_codes::hash_one(pt.clone(), state.password_pepper.clone()).await;
        drop(permit);
        hashes.push(h.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?);
    }
    state
        .users
        .replace_backup_codes(session.user_id, &hashes)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    Ok(Json(RegenerateResponse {
        backup_codes: plaintexts
            .into_iter()
            .map(|s| s.expose_secret().to_string())
            .collect(),
    }))
}

/// Verify either a 6-digit TOTP code OR a 10-char backup code. Length-
/// discriminated. Consumes the backup-code slot on success.
///
/// Returns `Ok(true)` on match, `Ok(false)` on mismatch.
pub(crate) async fn verify_code_any(
    state: &AppState,
    user_id: common::ids::UserId,
    code: &str,
) -> Result<bool, AppError> {
    let trimmed = code.trim();
    if trimmed.len() == 6 && trimmed.chars().all(|c| c.is_ascii_digit()) {
        let user = state
            .users
            .find_by_id(user_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
            .ok_or(AppError::Unauthorized)?;
        let Some(blob) = state
            .users
            .get_totp_secret(user_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        else {
            return Ok(false);
        };
        let secret = totp_at_rest::decrypt(&blob, &state.totp_key)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        let totp_inst = totp::build(&secret, TOTP_ISSUER, &user.email)
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        Ok(
            totp::verify(&totp_inst, trimmed)
                .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?,
        )
    } else if trimmed.len() == backup_codes::CODE_LEN {
        // Backup-code path: hash-verify each non-zeroed slot; on hit, CAS
        // the slot to all-zero.
        let slots = state
            .users
            .get_backup_codes(user_id)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
        for (i, slot) in slots.iter().enumerate() {
            if backup_codes::is_zeroed_slot(slot) {
                continue;
            }
            // Verify under the semaphore (Argon2id cost).
            let permit = state
                .hash_semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
            let ok = backup_codes::verify_one(
                SecretString::from(trimmed.to_string()),
                slot.clone(),
                state.password_pepper.clone(),
            )
            .await;
            drop(permit);
            if ok.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))? {
                let zeroed = backup_codes::zeroed_slot();
                let consumed = state
                    .users
                    .consume_backup_code_slot(user_id, i as i32, slot, &zeroed)
                    .await
                    .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
                // If CAS failed, another concurrent verify already used
                // this code. Treat as a miss to avoid double-consume.
                return Ok(consumed);
            }
        }
        Ok(false)
    } else {
        Ok(false)
    }
}
