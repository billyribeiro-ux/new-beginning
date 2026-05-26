//! Pluggable readiness checks.
//!
//! BACKEND.md §15 says `/readyz` returns 200 only when DB and Stripe checks
//! both pass. PR #1 has neither; later PRs register their checks here via
//! `AppState::with_readiness_check`. Empty registry → 200 with `checks: {}`,
//! which is the truthful answer until a real check is added.

use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait ReadinessCheck: Send + Sync {
    /// Short, fixed identifier used as the JSON key and metric label.
    /// Must be safe for both: keep it `snake_case`, no user input.
    fn name(&self) -> &'static str;

    /// Run the check. Implementations must enforce their own timeout so a
    /// hung dependency doesn't stall the whole readiness probe.
    async fn check(&self) -> Result<(), String>;
}

pub type DynReadinessCheck = Arc<dyn ReadinessCheck>;

/// Registry of readiness checks. Cheap to clone (`Arc<Vec<…>>`).
#[derive(Clone, Default)]
pub struct ReadinessRegistry {
    checks: Arc<Vec<DynReadinessCheck>>,
}

impl ReadinessRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, check: DynReadinessCheck) -> Self {
        Arc::make_mut(&mut self.checks).push(check);
        self
    }

    pub fn checks(&self) -> &[DynReadinessCheck] {
        &self.checks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysOk;
    #[async_trait]
    impl ReadinessCheck for AlwaysOk {
        fn name(&self) -> &'static str {
            "always_ok"
        }
        async fn check(&self) -> Result<(), String> {
            Ok(())
        }
    }

    struct AlwaysFails;
    #[async_trait]
    impl ReadinessCheck for AlwaysFails {
        fn name(&self) -> &'static str {
            "always_fails"
        }
        async fn check(&self) -> Result<(), String> {
            Err("nope".into())
        }
    }

    #[tokio::test]
    async fn registry_runs_all_checks() {
        let r = ReadinessRegistry::new()
            .with(Arc::new(AlwaysOk))
            .with(Arc::new(AlwaysFails));
        assert_eq!(r.checks().len(), 2);

        let mut results = Vec::new();
        for c in r.checks() {
            results.push((c.name(), c.check().await));
        }
        assert_eq!(results[0].0, "always_ok");
        assert!(results[0].1.is_ok());
        assert_eq!(results[1].0, "always_fails");
        assert!(results[1].1.is_err());
    }

    #[tokio::test]
    async fn empty_registry_has_no_checks() {
        let r = ReadinessRegistry::new();
        assert!(r.checks().is_empty());
    }
}
