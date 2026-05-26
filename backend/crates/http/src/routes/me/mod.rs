//! `/v1/me/*` — caller-owned profile + sessions.
//!
//! BACKEND.md §19 / §21 row 4. Every route requires the `AuthSession`
//! extractor; the router applies `require_auth` to the whole sub-tree in
//! `app::build_router`.

use axum::routing::{delete, get, post};
use axum::Router;

use crate::state::AppState;

pub mod account;
pub mod change_email;
pub mod change_password;
pub mod profile;
pub mod sessions;
pub mod twofa;

/// Returns an un-stated router; the caller (`app::build_router`) wraps it in
/// `require_auth` middleware so handlers only see authenticated requests.
pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/v1/me",
            get(profile::get_me)
                .patch(profile::patch_me)
                .delete(account::delete_me),
        )
        .route("/v1/me/export", get(account::export))
        .route("/v1/me/change-email", post(change_email::handler))
        .route("/v1/me/change-password", post(change_password::handler))
        .route("/v1/me/sessions", get(sessions::list))
        .route("/v1/me/sessions/{id}", delete(sessions::revoke))
        .route(
            "/v1/me/sessions/revoke-others",
            post(sessions::revoke_others),
        )
        .route("/v1/me/2fa/enable", post(twofa::enable))
        .route("/v1/me/2fa/confirm", post(twofa::confirm))
        .route("/v1/me/2fa/disable", post(twofa::disable))
        .route(
            "/v1/me/2fa/backup-codes/regenerate",
            post(twofa::regenerate),
        )
}

/// Shared user payload returned by `GET /v1/me` and the patch response.
/// Kept in this module (rather than reusing `auth/signup::UserPayload`) so
/// the surface here can evolve independently — e.g. PR #8 will add
/// `subscription` + `entitlements` fields.
#[derive(Debug, serde::Serialize)]
pub struct MePayload {
    pub id: String,
    pub email: String,
    pub name: String,
    pub headline: Option<String>,
    pub timezone: String,
    pub language: String,
    pub role: String,
    pub email_verified_at: Option<String>,
    pub created_at: String,
}

impl MePayload {
    pub fn from_user(u: &storage::User) -> Self {
        Self {
            id: u.id.to_string(),
            email: u.email.clone(),
            name: u.name.clone(),
            headline: u.headline.clone(),
            timezone: u.timezone.clone(),
            language: u.language.clone(),
            role: u.role.clone(),
            email_verified_at: u.email_verified_at.map(|t| {
                t.format(&time::format_description::well_known::Rfc3339)
                    .unwrap_or_default()
            }),
            created_at: u
                .created_at
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        }
    }
}
