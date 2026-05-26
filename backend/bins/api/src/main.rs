//! TradeFlex API binary.
//!
//! BACKEND.md §1.10 + §6. PR #3 adds: required `SERVICE_TOKEN`,
//! `AUTH_COOKIE_KEY`, `AUTH_PASSWORD_PEPPER`. The binary aborts boot if any
//! is missing — there is deliberately no fallback (no "dev secret if unset"
//! footgun).

use anyhow::Context;
use common::alerts::LogAlertSink;
use common::config::Config;
use http_crate::{build_router, AppState};
use r2_client::{ObjectStore, R2Client, RecordingObjectStore};
use secrecy::SecretString;
use std::net::SocketAddr;
use std::sync::Arc;
use storage::readiness::DbReadinessCheck;
use storage::{
    AuditRepo, ContactRepo, CourseProgressRepo, DownloadGrantsRepo, DownloadsCatalogRepo,
    EmailVerificationsRepo, EnrollmentsRepo, InvoicesRepo, LeadsRepo, LicensesRepo,
    NotificationPreferencesRepo, NotificationsRepo, OrdersRepo, PlansRepo, ProductsRepo,
    SessionsRepo, StripeEventsRepo, SubscriptionsRepo, UsersRepo,
};
use stripe_client::{StripeApi, StripeClient};
use tokio::net::TcpListener;
use tokio::signal::unix::{signal, SignalKind};

const SERVICE_NAME: &str = "tradeflex-api";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(Config::from_env()?);
    let _obs = observability::init(&config, SERVICE_NAME)?;

    // BACKEND.md §13: required secrets fail fast at boot, not at first use.
    let cookie_key: SecretString = config
        .auth_cookie_key
        .clone()
        .context("AUTH_COOKIE_KEY is required (BACKEND.md §13)")?;
    let password_pepper: SecretString = config
        .auth_password_pepper
        .clone()
        .context("AUTH_PASSWORD_PEPPER is required (BACKEND.md §13)")?;
    let totp_key: SecretString = config
        .auth_totp_key
        .clone()
        .context("AUTH_TOTP_KEY is required (BACKEND.md §1.2 / §7.3)")?;
    if config.service_token.is_none() {
        anyhow::bail!("SERVICE_TOKEN is required (BACKEND.md §1.1)");
    }
    let stripe_secret_key: SecretString = config
        .stripe_secret_key
        .clone()
        .context("STRIPE_SECRET_KEY is required (BACKEND.md §1.3)")?;
    let stripe_webhook_secret: SecretString = config
        .stripe_webhook_secret
        .clone()
        .context("STRIPE_WEBHOOK_SECRET is required (BACKEND.md §1.3)")?;

    tracing::info!(
        service = SERVICE_NAME,
        env = %config.env,
        bind = %config.bind_addr,
        metrics_bind = %config.metrics_bind,
        ip_allowlist = config.service_token_ip_allowlist.len(),
        "starting api binary",
    );

    // BACKEND.md §22 rule 15: migrations are forward-only and applied at boot.
    let pool = storage::build_pool(&config)
        .await
        .context("build postgres pool")?;
    storage::migrate::run(&pool)
        .await
        .context("run migrations")?;

    let alerts = Arc::new(LogAlertSink);
    let users = UsersRepo::new(pool.clone());
    let sessions = SessionsRepo::new(pool.clone(), config.auth_session_ttl_days as i64);
    let email_verifications = EmailVerificationsRepo::new(pool.clone());
    let products = ProductsRepo::new(pool.clone());
    let plans = PlansRepo::new(pool.clone());
    let leads = LeadsRepo::new(pool.clone());
    let contact = ContactRepo::new(pool.clone());
    let orders = OrdersRepo::new(pool.clone());
    let stripe_events = StripeEventsRepo::new(pool.clone());
    let enrollments = EnrollmentsRepo::new(pool.clone());
    let licenses = LicensesRepo::new(pool.clone());
    let invoices = InvoicesRepo::new(pool.clone());
    let subscriptions = SubscriptionsRepo::new(pool.clone());
    let audit = AuditRepo::new(pool.clone());
    let downloads_catalog = DownloadsCatalogRepo::new(pool.clone());
    let download_grants = DownloadGrantsRepo::new(pool.clone());
    let course_progress = CourseProgressRepo::new(pool.clone());
    let notifications = NotificationsRepo::new(pool.clone());
    let notification_prefs = NotificationPreferencesRepo::new(pool.clone());

    let stripe: Arc<dyn StripeApi> = Arc::new(
        StripeClient::new(stripe_secret_key, config.stripe_api_version.clone())
            .context("build StripeClient")?,
    );

    // R2 / S3 object store. The four R2_* env vars are all required for
    // the real client; if any is missing we fall back to an in-memory
    // `RecordingObjectStore` so local dev (without R2 creds) can still
    // boot. The fallback logs a WARN — it must never be silently used
    // in production. BACKEND.md §1.4 / §9.
    let r2: Arc<dyn ObjectStore> = match (
        config.r2_endpoint.as_deref(),
        config.r2_bucket.as_deref(),
        config.r2_access_key_id.clone(),
        config.r2_secret_access_key.clone(),
    ) {
        (Some(ep), Some(b), Some(ak), Some(sk)) => {
            Arc::new(R2Client::new(ep, b, ak, sk).context("build R2Client")?)
        }
        _ => {
            if config.is_production() {
                anyhow::bail!(
                    "R2_ENDPOINT, R2_BUCKET, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY are required in production (BACKEND.md §9)"
                );
            }
            tracing::warn!(
                "R2_* env vars missing — using RecordingObjectStore (dev-only fallback)"
            );
            Arc::new(RecordingObjectStore::new())
        }
    };

    let state = AppState::new(
        config.clone(),
        alerts,
        pool.clone(),
        users,
        sessions,
        email_verifications,
        products,
        plans,
        leads,
        contact,
        orders,
        stripe_events,
        enrollments,
        licenses,
        invoices,
        subscriptions,
        audit,
        downloads_catalog,
        download_grants,
        course_progress,
        notifications,
        notification_prefs,
        stripe,
        r2,
        cookie_key,
        password_pepper,
        totp_key,
        stripe_webhook_secret,
    )
    .with_readiness_check(Arc::new(DbReadinessCheck::new(pool.clone())));

    let app = build_router(state);
    let listener = TcpListener::bind(config.bind_addr).await?;
    tracing::info!(addr = %config.bind_addr, "listening");

    // ConnectInfo<SocketAddr> requires `into_make_service_with_connect_info`
    // (service_token middleware reads the peer IP for the allowlist check).
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    tracing::info!("api binary shut down cleanly");
    Ok(())
}

async fn shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    tokio::select! {
        _ = sigterm.recv() => tracing::info!("received SIGTERM, shutting down"),
        _ = sigint.recv() => tracing::info!("received SIGINT, shutting down"),
    }
}
