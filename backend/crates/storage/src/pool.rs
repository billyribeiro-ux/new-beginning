//! Postgres connection-pool init.
//!
//! BACKEND.md §13: `DATABASE_MAX_CONN` defaults to 20, `DATABASE_MIN_CONN`
//! to 2. The pool is created once at startup and shared via `Arc` (sqlx's
//! `PgPool` is `Arc`-internal).
//!
//! Connect timeout is hard-coded short (5 s) so a misconfigured DB URL fails
//! boot fast instead of hanging behind a 30-second TCP handshake — CLAUDE.md
//! external-client rule.

use common::config::Config;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("DATABASE_URL is required but unset")]
    MissingUrl,

    #[error("postgres connect: {0}")]
    Connect(#[from] sqlx::Error),
}

pub async fn build_pool(config: &Config) -> Result<PgPool, PoolError> {
    let url = config
        .database_url
        .as_ref()
        .ok_or(PoolError::MissingUrl)?
        .expose_secret()
        .to_owned();

    let max = config.database_max_conn;
    let min = config.database_min_conn;

    tracing::info!(max_conn = max, min_conn = min, "opening postgres pool");

    let pool = PgPoolOptions::new()
        .max_connections(max)
        .min_connections(min)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&url)
        .await?;

    Ok(pool)
}
