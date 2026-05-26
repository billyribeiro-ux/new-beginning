//! `POST /v1/me/change-email` — stage a verification.
//!
//! Step 1 of a two-step flow: stages a row in `email_verifications`
//! (`kind='email_change'`, `new_email` populated) and logs the verification
//! token. **Step 2 (the actual email change) ships when the mailer crate
//! lands and the `/v1/auth/verify-email` endpoint can consume the row.**
//!
//! Until then, the dev workflow is: tail the api logs for the line tagged
//! `EMAIL_STUB email_change`, grab the token, and hit
//! `POST /v1/auth/verify-email` (PR-X).

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use common::auth::verification_token;
use common::error::AppError;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use storage::VerificationKind;
use time::Duration;
use validator::Validate;

use crate::auth::AuthSession;
use crate::state::AppState;

const VERIFICATION_TTL: Duration = Duration::hours(1);

#[derive(Debug, Deserialize, Validate)]
pub struct ChangeEmailRequest {
    #[validate(email)]
    pub new_email: String,
}

#[derive(Debug, Serialize)]
pub struct ChangeEmailResponse {
    /// "verification_required" today. Future: "no_change" when the request
    /// matches the current email.
    pub status: &'static str,
    /// ISO-8601 expiry of the staged verification.
    pub expires_at: String,
}

pub async fn handler(
    State(state): State<AppState>,
    session: AuthSession,
    Json(body): Json<ChangeEmailRequest>,
) -> Result<(StatusCode, Json<ChangeEmailResponse>), AppError> {
    body.validate()
        .map_err(|_| AppError::Validation("new_email must be a valid email".into()))?;

    let new_email = body.new_email.trim().to_string();

    // If the user supplied their current email, treat as a no-op success.
    // The DB unique constraint on `users.email` is case-insensitive so we
    // compare via the storage layer instead of lowercasing here.
    let user = state
        .users
        .find_by_id(session.user_id)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
        .ok_or(AppError::NotFound)?;
    if user.email.eq_ignore_ascii_case(&new_email) {
        return Ok((
            StatusCode::OK,
            Json(ChangeEmailResponse {
                status: "no_change",
                expires_at: time::OffsetDateTime::now_utc()
                    .format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default(),
            }),
        ));
    }

    // Optimistic uniqueness check — the DB enforces the constraint later
    // when the email change is committed via verify-email. Returning 409
    // here gives the user immediate feedback.
    if let Some(_existing) = state
        .users
        .find_by_email(&new_email)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?
    {
        return Err(AppError::Conflict("email already in use".into()));
    }

    let token = verification_token::issue();
    let row = state
        .email_verifications
        .create(
            session.user_id,
            VerificationKind::EmailChange,
            &token.hash,
            Some(&new_email),
            VERIFICATION_TTL,
        )
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    // BACKEND_NOTES PR #4 decision: log the token in lieu of a real email
    // until the mailer crate ships. The `EMAIL_STUB` prefix is grep-able.
    tracing::warn!(
        verification_id = %row.id,
        user_id = %session.user_id,
        new_email = %new_email,
        kind = "email_change",
        token = %token.plaintext.expose_secret(),
        "EMAIL_STUB email_change — replace with mailer enqueue in PR-X",
    );

    Ok((
        StatusCode::ACCEPTED,
        Json(ChangeEmailResponse {
            status: "verification_required",
            expires_at: row
                .expires_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        }),
    ))
}
