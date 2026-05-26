//! `POST /v1/auth/logout` — revokes the current session.
//!
//! Returns 204 on revoke. Also returns 204 if the cookie is missing or
//! already-revoked — the property the BFF wants is "after this call, the
//! caller has no session", which is true in all three cases.

use axum::extract::State;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum_extra::extract::CookieJar;
use common::auth::session_cookie::{self, COOKIE_NAME};
use common::error::AppError;

use crate::state::AppState;

pub async fn handler(State(state): State<AppState>, jar: CookieJar) -> Result<Response, AppError> {
    if let Some(cookie) = jar.get(COOKIE_NAME) {
        if let Ok(parsed) = session_cookie::verify(cookie.value(), &state.cookie_key) {
            // Look up by token hash to get the session id (cookie may have
            // lied about the id); revoke whatever the DB actually has.
            if let Ok(Some(active)) = state.sessions.load_by_token_hash(&parsed.token_hash).await {
                let _ = state.sessions.revoke(active.id).await;
            }
        }
    }

    // Clear the cookie client-side regardless. `Max-Age=0` is the standard
    // delete pattern; the actual value is ignored by the browser.
    let mut resp = StatusCode::NO_CONTENT.into_response();
    let clear = format!("{COOKIE_NAME}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0");
    if let Ok(v) = HeaderValue::from_str(&clear) {
        resp.headers_mut().insert(header::SET_COOKIE, v);
    }
    Ok(resp)
}
