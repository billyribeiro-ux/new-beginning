//! `X-Service-Token` gate.
//!
//! BACKEND.md §1.1 + §6: the SvelteKit BFF authenticates to Rust with a
//! single header. The header name on the wire is `x-service-token`; the
//! constant is the lowercase-hyphen form so every reference (here, in
//! `BACKEND.md`, and in `src/lib/server/rust-client.ts` once PR #20 lands)
//! agrees.
//!
//! Compare is constant-time. An optional CIDR allowlist gates by source IP;
//! when unset the IP check is skipped (local dev), when set every request
//! must originate from an allowed network.

use axum::{
    body::Body,
    extract::{ConnectInfo, State},
    http::{HeaderName, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use ipnet::IpNet;
use secrecy::ExposeSecret;
use std::net::SocketAddr;
use subtle::ConstantTimeEq;

use crate::state::AppState;

pub const HEADER: HeaderName = HeaderName::from_static("x-service-token");

pub async fn verify(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request<Body>,
    next: Next,
) -> Response {
    // Token check ----------------------------------------------------------
    let Some(expected) = state.config.service_token.as_ref() else {
        // Misconfiguration — fail closed. The api binary's main() refuses
        // to boot when SERVICE_TOKEN is unset; this branch is here so a
        // hypothetical future caller of `build_router` without main() also
        // fails closed instead of opening up.
        tracing::error!("service_token middleware: SERVICE_TOKEN unset; rejecting all requests");
        return reject(
            StatusCode::INTERNAL_SERVER_ERROR,
            "service auth misconfigured",
        );
    };

    let presented = req.headers().get(&HEADER).and_then(|v| v.to_str().ok());
    let presented = match presented {
        Some(s) => s,
        None => {
            tracing::warn!(peer = %peer.ip(), "missing x-service-token");
            return reject(StatusCode::UNAUTHORIZED, "missing service token");
        }
    };

    let exp = expected.expose_secret().as_bytes();
    let got = presented.as_bytes();
    // Length mismatch is rejected explicitly to keep the constant-time
    // compare valid only when both sides agree on length.
    if exp.len() != got.len() || !bool::from(exp.ct_eq(got)) {
        tracing::warn!(peer = %peer.ip(), "x-service-token mismatch");
        return reject(StatusCode::UNAUTHORIZED, "invalid service token");
    }

    // IP allowlist check (skipped when list is empty) ----------------------
    if !state.config.service_token_ip_allowlist.is_empty() {
        let peer_ip = peer.ip();
        let allowed = state
            .config
            .service_token_ip_allowlist
            .iter()
            .any(|cidr: &IpNet| cidr.contains(&peer_ip));
        if !allowed {
            tracing::warn!(peer = %peer_ip, "service token presented from non-allowlisted ip");
            return reject(StatusCode::FORBIDDEN, "source not allowlisted");
        }
    }

    next.run(req).await
}

fn reject(status: StatusCode, msg: &'static str) -> Response {
    (
        status,
        axum::Json(serde_json::json!({
            "error": { "code": "service_auth", "message": msg }
        })),
    )
        .into_response()
}
