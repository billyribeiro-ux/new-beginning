//! `POST /v1/auth/signup`.
//!
//! Mirrors STACK.md / `src/lib/utils/validators.ts` shape:
//! - `name`: 1–80 chars
//! - `email`: valid
//! - `password`: 8–128 chars, ≥1 upper/lower/digit  (enforced server-side)
//! - `terms`: must be `true`
//!
//! On success: 201 with the user payload + sets the `tfx_session` cookie.

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{http, Json};
use common::auth::password::hash_password;
use common::auth::session_cookie;
use common::error::AppError;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct SignupRequest {
    #[validate(length(min = 1, max = 80))]
    pub name: String,

    #[validate(email)]
    pub email: String,

    #[validate(length(min = 8, max = 128))]
    pub password: String,

    pub terms: bool,
}

#[derive(Debug, Serialize)]
pub struct SignupResponse {
    pub user: UserPayload,
}

#[derive(Debug, Serialize)]
pub struct UserPayload {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Json(body): Json<SignupRequest>,
) -> Result<Response, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(format_validation(&e)))?;
    if !body.terms {
        return Err(AppError::Validation("terms must be accepted".into()));
    }
    enforce_password_strength(&body.password)?;

    // Hash under the semaphore: BACKEND.md §7.2 anti-OOM cap.
    let permit = state
        .hash_semaphore
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| AppError::Internal(anyhow::anyhow!("hash semaphore closed")))?;
    let hash_result = hash_password(
        SecretString::from(body.password.clone()),
        state.password_pepper.clone(),
    )
    .await;
    drop(permit);
    let hash = hash_result.map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let user = state
        .users
        .create(&body.email, &body.name, &hash)
        .await
        .map_err(|e| match e {
            storage::UsersError::DuplicateEmail => {
                AppError::Conflict("email already in use".into())
            }
            other => AppError::Internal(anyhow::anyhow!(other)),
        })?;

    // Mint the session cookie + insert the row.
    let issued = session_cookie::issue(&state.cookie_key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;
    state
        .sessions
        .create(issued.session_id, user.id, &issued.token_hash, None, None)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!(e)))?;

    let mut resp = (
        StatusCode::CREATED,
        Json(SignupResponse {
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

/// Hand-rolled strength check matching the frontend Zod shape.
/// `validator::ContainsCharacterType` would be overkill for three rules.
fn enforce_password_strength(pw: &str) -> Result<(), AppError> {
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

fn format_validation(e: &validator::ValidationErrors) -> String {
    // Compact one-line summary for the user. Detailed field map could come
    // later if the BFF wants per-field messages.
    let mut msgs: Vec<String> = e
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| errs.iter().map(move |er| format!("{field}: {}", er.code)))
        .collect();
    msgs.sort();
    msgs.join(", ")
}

pub(crate) fn format_cookie(state: &AppState, value: &str) -> Result<HeaderValue, AppError> {
    let max_age_secs = state.config.auth_session_ttl_days as i64 * 86_400;
    let domain_part = state
        .config
        .auth_cookie_domain
        .as_deref()
        .map(|d| format!("; Domain={d}"))
        .unwrap_or_default();
    let secure_part = if state.config.is_production() {
        "; Secure"
    } else {
        // SameSite=Lax requires Secure on cross-site requests, but on localhost
        // browsers permit it without. Production always sets Secure.
        ""
    };
    let cookie = format!(
        "{name}={value}; HttpOnly{secure_part}; SameSite=Lax; Path=/; Max-Age={max_age_secs}{domain_part}",
        name = session_cookie::COOKIE_NAME,
    );
    HeaderValue::from_str(&cookie)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("cookie format: {e}")))
}

#[allow(unused_imports)]
use http::header as _http_header;
