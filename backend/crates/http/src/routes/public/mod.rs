//! `/v1/public/*` — anonymous-callable surface.
//!
//! BACKEND.md §19 + §21 row 6. The Rust API still gates every request with
//! `X-Service-Token` (BACKEND.md §1.1); "public" here means "the BFF doesn't
//! need a session cookie before forwarding". Each route consumes a per-IP
//! bucket from `RateLimiterSet` (BACKEND.md §12).

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

pub mod contact;
pub mod leads;
pub mod plans;
pub mod products;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/public/products", get(products::list))
        .route("/v1/public/products/{slug}", get(products::detail))
        .route("/v1/public/plans", get(plans::list))
        .route("/v1/public/leads", post(leads::capture))
        .route("/v1/public/contact", post(contact::submit))
}
