//! Route modules. Each module owns a router builder; `app::build_router`
//! merges them.

pub mod admin;
pub mod auth;
pub mod billing;
pub mod checkout;
pub mod courses;
pub mod downloads;
pub mod health;
pub mod indicators;
pub mod me;
pub mod notifications;
pub mod public;
pub mod subscription;
pub mod webhooks;
