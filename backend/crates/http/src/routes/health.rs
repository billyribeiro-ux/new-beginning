//! `/healthz` and `/readyz` handlers.
//!
//! BACKEND.md §15:
//! - `/healthz`: 200 unconditionally if the process is alive.
//! - `/readyz`: 200 only if every registered readiness check passes. The set
//!   of checks is empty in PR #1 and expands additively (PR #2 = DB, PR #8 =
//!   Stripe). A failing check fires `AlertKind::ReadinessDegraded`.

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use common::alerts::AlertKind;
use serde_json::{json, Map, Value};

use crate::state::AppState;

pub async fn live() -> impl IntoResponse {
    (StatusCode::OK, Json(json!({ "status": "ok" })))
}

pub async fn ready(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks: Map<String, Value> = Map::new();
    let mut all_ok = true;

    for check in state.readiness.checks() {
        let name = check.name();
        match check.check().await {
            Ok(()) => {
                checks.insert(name.into(), Value::String("ok".into()));
            }
            Err(reason) => {
                checks.insert(name.into(), Value::String(format!("fail: {reason}")));
                all_ok = false;
                // BACKEND.md §15: a failing check fires ReadinessDegraded.
                state
                    .alerts
                    .fire_async(AlertKind::ReadinessDegraded { check: name });
            }
        }
    }

    let status = if all_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "status": if all_ok { "ready" } else { "degraded" },
            "checks": Value::Object(checks),
        })),
    )
}

#[cfg(test)]
// `Jail::expect_with`'s closure returns `Result<(), figment::Error>` — the
// boxed alternative isn't applicable.
#[allow(clippy::result_large_err)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::{body::Body, http::Request, routing::get, Router};
    use common::alerts::{LogAlertSink, RecordingSink};
    use common::config::Config;
    use common::readiness::ReadinessCheck;
    use figment::{providers::Env, Figment, Jail};
    use http_body_util::BodyExt;
    use secrecy::SecretString;
    use sqlx::PgPool;
    use std::sync::Arc;
    use tower::ServiceExt;

    fn test_config() -> Arc<Config> {
        let mut cfg: Option<Config> = None;
        Jail::expect_with(|jail| {
            jail.clear_env();
            jail.set_env("SERVICE_TOKEN", "test-token");
            cfg = Some(Figment::new().merge(Env::raw()).extract().unwrap());
            Ok(())
        });
        Arc::new(cfg.unwrap())
    }

    async fn test_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://tradeflex:tradeflex@127.0.0.1:5435/tradeflex".into());
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(&url)
            .await
            .expect("test DB pool — is `just up` running?")
    }

    async fn test_state(alerts: Arc<dyn common::alerts::AlertSink>) -> AppState {
        let pool = test_pool().await;
        let stripe: Arc<dyn stripe_client::StripeApi> =
            Arc::new(stripe_client::RecordingStripeApi::new());
        let r2: Arc<dyn r2_client::ObjectStore> = Arc::new(r2_client::RecordingObjectStore::new());
        AppState::new(
            test_config(),
            alerts,
            pool.clone(),
            storage::UsersRepo::new(pool.clone()),
            storage::SessionsRepo::new(pool.clone(), 30),
            storage::EmailVerificationsRepo::new(pool.clone()),
            storage::ProductsRepo::new(pool.clone()),
            storage::PlansRepo::new(pool.clone()),
            storage::LeadsRepo::new(pool.clone()),
            storage::ContactRepo::new(pool.clone()),
            storage::OrdersRepo::new(pool.clone()),
            storage::StripeEventsRepo::new(pool.clone()),
            storage::EnrollmentsRepo::new(pool.clone()),
            storage::LicensesRepo::new(pool.clone()),
            storage::InvoicesRepo::new(pool.clone()),
            storage::SubscriptionsRepo::new(pool.clone()),
            storage::AuditRepo::new(pool.clone()),
            storage::DownloadsCatalogRepo::new(pool.clone()),
            storage::DownloadGrantsRepo::new(pool.clone()),
            storage::CourseProgressRepo::new(pool.clone()),
            storage::NotificationsRepo::new(pool.clone()),
            storage::NotificationPreferencesRepo::new(pool),
            stripe,
            r2,
            SecretString::from("test-cookie-key-32-bytes-long-aaaa".to_string()),
            SecretString::from("test-pepper".to_string()),
            SecretString::from("0123456789abcdef0123456789abcdef".to_string()),
            SecretString::from("whsec_test_pr7_health_helper".to_string()),
        )
    }

    fn router(state: AppState) -> Router {
        Router::new()
            .route("/healthz", get(live))
            .route("/readyz", get(ready))
            .with_state(state)
    }

    #[tokio::test]
    async fn healthz_always_200() {
        let state = test_state(Arc::new(LogAlertSink)).await;
        let resp = router(state)
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "ok");
    }

    #[tokio::test]
    async fn readyz_empty_registry_returns_200() {
        let state = test_state(Arc::new(LogAlertSink)).await;
        let resp = router(state)
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "ready");
        assert!(v["checks"].as_object().unwrap().is_empty());
    }

    struct FailingCheck;
    #[async_trait]
    impl ReadinessCheck for FailingCheck {
        fn name(&self) -> &'static str {
            "db"
        }
        async fn check(&self) -> Result<(), String> {
            Err("connection refused".into())
        }
    }

    #[tokio::test]
    async fn readyz_failing_check_returns_503_and_fires_alert() {
        let recorder = Arc::new(RecordingSink::new());
        let state = test_state(recorder.clone())
            .await
            .with_readiness_check(Arc::new(FailingCheck));

        let resp = router(state)
            .oneshot(
                Request::builder()
                    .uri("/readyz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let v: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(v["status"], "degraded");
        assert!(v["checks"]["db"].as_str().unwrap().contains("fail"));
        assert_eq!(recorder.recorded().len(), 1);
        assert_eq!(
            recorder.recorded()[0],
            AlertKind::ReadinessDegraded { check: "db" }
        );
    }
}
