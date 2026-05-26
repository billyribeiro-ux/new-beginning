//! `AuthSession` extractor.
//!
//! BACKEND.md §7.4: reads the `tfx_session` cookie, HMAC-verifies it, looks
//! up the row in `sessions`, and yields `(user_id, session_id, role)`. Rejects
//! with `AppError::Unauthorized` on any failure; the exact reason is logged
//! but never returned to the client.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use common::auth::session_cookie::{verify, COOKIE_NAME};
use common::error::AppError;
use common::ids::{SessionId, UserId};

use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub role: Role,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Member,
    Admin,
}

impl Role {
    fn from_db(s: &str) -> Self {
        match s {
            "admin" => Role::Admin,
            _ => Role::Member,
        }
    }

    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin)
    }
}

impl FromRequestParts<AppState> for AuthSession {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        // Extract cookies. CookieJar's extractor cannot fail on its own.
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar.get(COOKIE_NAME).ok_or(AppError::Unauthorized)?;

        // Verify HMAC + parse session id and token hash.
        let parsed = verify(cookie.value(), &state.cookie_key).map_err(|e| {
            tracing::debug!(error = ?e, "session cookie verify failed");
            AppError::Unauthorized
        })?;

        // Look up the row + update last_seen_at in one query.
        let active = state
            .sessions
            .load_by_token_hash(&parsed.token_hash)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "session lookup failed");
                AppError::Internal(anyhow::anyhow!("session lookup"))
            })?
            .ok_or(AppError::Unauthorized)?;

        // Cookie's claimed session id MUST match the DB row reached via the
        // token hash. If they disagree, the cookie was forged from a stolen
        // token + a guess at the id — reject.
        if active.id != parsed.session_id {
            tracing::warn!(
                cookie_session_id = %parsed.session_id,
                db_session_id = %active.id,
                "session id mismatch between cookie and DB",
            );
            return Err(AppError::Unauthorized);
        }

        // Pull the role separately — it's not on `ActiveSession` because the
        // sessions table doesn't store it; we go to `users`.
        let user = state
            .users
            .find_by_id(active.user_id)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "user lookup failed");
                AppError::Internal(anyhow::anyhow!("user lookup"))
            })?
            .ok_or(AppError::Unauthorized)?;

        Ok(AuthSession {
            user_id: active.user_id,
            session_id: active.id,
            role: Role::from_db(&user.role),
        })
    }
}

/// Convenience middleware: enforce auth on a route tree. Inserts `AuthSession`
/// into the request extensions; downstream handlers can extract it without
/// re-running the work.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    let (mut parts, body) = req.into_parts();
    let session = AuthSession::from_request_parts(&mut parts, &state).await?;
    req = axum::http::Request::from_parts(parts, body);
    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}

pub async fn require_admin(
    State(state): State<AppState>,
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    let (mut parts, body) = req.into_parts();
    let session = AuthSession::from_request_parts(&mut parts, &state).await?;
    if !session.role.is_admin() {
        return Err(AppError::Forbidden);
    }
    req = axum::http::Request::from_parts(parts, body);
    req.extensions_mut().insert(session);
    Ok(next.run(req).await)
}

use axum::extract::State;
