//! HTTP-layer auth: middleware, extractors, login limiter.
//!
//! BACKEND.md §6 + §7. The reusable crypto primitives (hash/verify,
//! session-cookie format) live in `common::auth`; this module is the
//! axum-bound wiring.

pub mod auth_session;
pub mod service_token;

pub use auth_session::AuthSession;
