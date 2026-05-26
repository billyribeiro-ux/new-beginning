//! Migration runner.
//!
//! BACKEND.md §1.6: forward-only migrations under `backend/migrations/`,
//! applied via `sqlx::migrate!`. Embedded at compile time so the binary is
//! self-contained — no need to ship the SQL alongside it.

use sqlx::PgPool;

/// Compile-time-embedded migrations. Path is relative to this file, walking
/// up to `backend/migrations/`.
static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    #[error("migrate: {0}")]
    Run(#[from] sqlx::migrate::MigrateError),
}

pub async fn run(pool: &PgPool) -> Result<(), MigrateError> {
    let before = MIGRATOR.iter().map(|m| m.version).collect::<Vec<_>>();
    tracing::info!(migrations = before.len(), "running migrations");
    MIGRATOR.run(pool).await?;
    tracing::info!("migrations applied");
    Ok(())
}
