//! TradeFlex shared primitives.
//!
//! Per BACKEND.md §1, this crate is infra-free: no tokio, no sqlx, no axum
//! handlers — only the building blocks (Money, AppError, AlertSink, Config,
//! typed ids) that every other crate depends on.

pub mod alerts;
pub mod auth;
pub mod config;
pub mod deserialize;
pub mod error;
pub mod ids;
pub mod money;
pub mod readiness;
pub mod request_id;
