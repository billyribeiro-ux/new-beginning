//! `/v1/auth/*` route handlers.
//!
//! BACKEND.md §19 + §21 row 3:
//! - `POST /v1/auth/signup` → 201, sets cookie.
//! - `POST /v1/auth/login` → 200, sets cookie.
//! - `POST /v1/auth/logout` → 204, revokes the current session.
//!
//! Forgot-password / reset-password / verify-email live in PR #4.

use axum::routing::post;
use axum::Router;

use crate::state::AppState;

pub mod login;
pub mod login_totp;
pub mod logout;
pub mod signup;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/auth/signup", post(signup::handler))
        .route("/v1/auth/login", post(login::handler))
        .route("/v1/auth/login/totp", post(login_totp::handler))
        .route("/v1/auth/logout", post(logout::handler))
}
