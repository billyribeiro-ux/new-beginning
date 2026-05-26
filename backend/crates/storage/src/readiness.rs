//! DB readiness check.
//!
//! BACKEND.md §15: `/readyz` returns 200 only if `SELECT 1` against the
//! pool completes in < 100 ms. Plugs into `common::readiness::ReadinessRegistry`
//! via `AppState::with_readiness_check` in the api binary.

use async_trait::async_trait;
use common::readiness::ReadinessCheck;
use sqlx::PgPool;
use std::time::Duration;

pub struct DbReadinessCheck {
    pool: PgPool,
    timeout: Duration,
}

impl DbReadinessCheck {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            timeout: Duration::from_millis(100),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

#[async_trait]
impl ReadinessCheck for DbReadinessCheck {
    fn name(&self) -> &'static str {
        "db"
    }

    async fn check(&self) -> Result<(), String> {
        let q = sqlx::query_scalar::<_, i32>("SELECT 1").fetch_one(&self.pool);
        match tokio::time::timeout(self.timeout, q).await {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(format!("query failed: {e}")),
            Err(_) => Err(format!("timeout after {:?}", self.timeout)),
        }
    }
}
