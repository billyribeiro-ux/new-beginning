//! HTTP layer for the api binary.
//!
//! BACKEND.md §6. PR #3 adds: `service_token` middleware, `AuthSession`
//! extractor, login limiter, signup/login/logout routes. Other routes land
//! in later PRs.

pub mod app;
pub mod auth;
pub mod jobs;
pub mod middleware;
pub mod routes;
pub mod state;

pub use app::build_router;
pub use common::readiness;
pub use state::AppState;
