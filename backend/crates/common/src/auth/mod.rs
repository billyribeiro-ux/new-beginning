//! Auth primitives — Argon2id hashing/verify with keyed pepper, opaque
//! session-cookie format.
//!
//! Lives in `common` so `storage`, `http`, and any future surface (worker,
//! admin CLI) can use the same primitives without dragging axum or sqlx into
//! each other's dep graphs.

pub mod backup_codes;
pub mod idempotency;
pub mod password;
pub mod pending_totp_token;
pub mod session_cookie;
pub mod totp;
pub mod totp_secret_at_rest;
pub mod verification_token;
