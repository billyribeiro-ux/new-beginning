//! Router composition.
//!
//! BACKEND.md §6 specifies the middleware order: CatchPanic → RequestId →
//! Trace → Timeout → BodyLimit → ServiceToken → Compression. PR #4 adds the
//! authenticated `/v1/me/*` sub-tree, gated by `require_auth` (defined in
//! `auth::auth_session`).
//!
//! `Router::layer` semantics: each `.layer(L)` applied LATER becomes MORE
//! OUTER. Layers below are written innermost-first; the resulting outside-in
//! stack matches the spec.

use axum::{http::StatusCode, middleware, routing::get, Router};
use std::time::Duration;
use tower_http::{
    catch_panic::CatchPanicLayer, compression::CompressionLayer, limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};

use crate::{
    auth::{
        auth_session::{require_admin, require_auth},
        service_token,
    },
    middleware::request_id,
    routes::{
        admin, auth as auth_routes, billing, checkout, courses, downloads, health, indicators, me,
        notifications, public as public_routes, subscription, webhooks,
    },
    state::AppState,
};
use axum::routing::post;

pub fn build_router(state: AppState) -> Router {
    // No-user-auth routes: health, auth endpoints, /v1/public/*, and the
    // Stripe webhook. All gated by the outer `X-Service-Token` middleware
    // (BACKEND.md §1.1). Stripe webhooks pass through the BFF too — the
    // BFF route `/api/webhooks/stripe` forwards the raw body + signature
    // header AND injects the service token; otherwise we'd have a public
    // Rust URL, which BACKEND.md §1.1 forbids.
    let public = Router::new()
        .route("/healthz", get(health::live))
        .route("/readyz", get(health::ready))
        .merge(auth_routes::router())
        .merge(public_routes::router())
        .route("/v1/webhooks/stripe", post(webhooks::stripe::handler));

    // Authenticated routes: caller-owned profile + sessions + checkout +
    // subscription read + billing portal.
    let authed = me::router()
        .route("/v1/checkout", post(checkout::handler))
        .route("/v1/subscription", get(subscription::get))
        .route("/v1/billing/portal", post(billing::portal::handler))
        .route("/v1/billing/invoices", get(billing::invoices::list))
        .route(
            "/v1/billing/invoices/{id}/pdf-url",
            get(billing::invoices::pdf_url),
        )
        .route("/v1/indicators", get(indicators::list))
        .route("/v1/indicators/{slug}/key", get(indicators::key_for_slug))
        .route(
            "/v1/indicators/{slug}/downloads",
            get(indicators::downloads_for_slug),
        )
        .route("/v1/downloads", get(downloads::list))
        .route("/v1/downloads/{download_id}/url", get(downloads::url))
        .route("/v1/courses", get(courses::list))
        .route("/v1/courses/{slug}", get(courses::player_state))
        .route("/v1/courses/{slug}/progress", post(courses::post_progress))
        .route(
            "/v1/courses/{slug}/lessons/{lesson_id}/notes",
            axum::routing::put(courses::put_notes),
        )
        .route("/v1/notifications", get(notifications::list))
        .route(
            "/v1/notifications/{id}/read",
            axum::routing::patch(notifications::mark_read),
        )
        .route(
            "/v1/notifications/mark-all-read",
            post(notifications::mark_all_read),
        )
        .route(
            "/v1/notifications/preferences",
            get(notifications::get_prefs).patch(notifications::patch_prefs),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), require_auth));

    // Admin sub-tree — gated by `require_admin` (which also runs auth).
    let admin = Router::new()
        .route("/v1/admin/stats", get(admin::kpis::get_stats))
        .route("/v1/admin/leads", get(admin::leads::list))
        .route("/v1/admin/messages", get(admin::messages::list))
        .route(
            "/v1/admin/messages/{id}",
            axum::routing::patch(admin::messages::patch),
        )
        .route("/v1/admin/products", get(admin::products::list))
        .route(
            "/v1/admin/products/{id}",
            axum::routing::patch(admin::products::patch),
        )
        .route("/v1/admin/customers", get(admin::customers::search))
        .route(
            "/v1/admin/customers/{id}/grant-entitlement",
            post(admin::customers::grant_entitlement),
        )
        .route("/v1/admin/orders/{id}/refund", post(admin::refund::trigger))
        .route_layer(middleware::from_fn_with_state(state.clone(), require_admin));

    public
        .merge(authed)
        .merge(admin)
        // Innermost first ↓
        .layer(CompressionLayer::new().gzip(true))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            service_token::verify,
        ))
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(15),
        ))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(request_id::layer))
        .layer(CatchPanicLayer::new())
        // ↑ Outermost
        .with_state(state)
}

// Integration tests for auth + me routes live in
// `crates/http/tests/*_integration.rs` (added per-PR) so they can boot a
// real Postgres and exercise the full request lifecycle.
