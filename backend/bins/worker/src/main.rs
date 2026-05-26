//! TradeFlex worker binary.
//!
//! BACKEND.md §1.7 + §21 PR #11. Long-running process that drives the
//! re-runnable jobs the api binary doesn't want in its request path:
//!
//! * `reconcile_stripe_events` — every 5 minutes, re-drives any
//!   `stripe_events` row with `processed_at IS NULL` older than 5 min.
//!   The api binary's webhook receiver inserts-then-dispatches in a tx;
//!   the worker's reconciliation is the backstop for the crash-mid-flight
//!   case (BACKEND.md §8.3).
//! * `fetch_invoice_pdfs` — every 5 minutes, lists invoices with
//!   `pdf_r2_key IS NULL`, fetches their PDF from Stripe, uploads to R2,
//!   and stamps the key. PR #10 added the storage + endpoint; this is
//!   the driver.
//!
//! The worker shares `AppState` construction with the api binary so every
//! repo + client wired into a route is reachable from a job. Jobs that
//! mutate Stripe state pass through the same `stripe-client` instance, so
//! Idempotency-Key headers are derived consistently.

use anyhow::Context;
use common::alerts::LogAlertSink;
use common::config::Config;
use http_crate::jobs::{
    fetch_invoice_pdfs::fetch_invoice_pdfs, reconcile_stripe_events::reconcile_stripe_events,
};
use http_crate::AppState;
use r2_client::{ObjectStore, R2Client, RecordingObjectStore};
use secrecy::SecretString;
use std::sync::Arc;
use std::time::Duration;
use storage::{
    AuditRepo, ContactRepo, CourseProgressRepo, DownloadGrantsRepo, DownloadsCatalogRepo,
    EmailVerificationsRepo, EnrollmentsRepo, InvoicesRepo, LeadsRepo, LicensesRepo,
    NotificationPreferencesRepo, NotificationsRepo, OrdersRepo, PlansRepo, ProductsRepo,
    SessionsRepo, StripeEventsRepo, SubscriptionsRepo, UsersRepo,
};
use stripe_client::{StripeApi, StripeClient};
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::interval;

const SERVICE_NAME: &str = "tradeflex-worker";

/// Sweep cadence. 5 min matches the `min_age_secs` floor for reconciliation
/// (BACKEND.md §8.3) and is small enough that invoice PDFs land quickly
/// after the webhook insert.
const SWEEP_INTERVAL: Duration = Duration::from_secs(300);

/// `min_age_secs` for reconciliation — only re-drive rows older than this.
/// Prevents a race between the api's webhook receiver mid-tx and the
/// worker's sweep.
const RECONCILE_MIN_AGE_SECS: i64 = 300;

/// Per-sweep cap. Plenty for a steady-state TradeFlex workload; if a
/// backlog ever exceeds this the loop just picks it up next tick.
const SWEEP_LIMIT: i64 = 100;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(Config::from_env()?);
    let _obs = observability::init(&config, SERVICE_NAME)?;

    // Same fail-fast secrets the api binary requires. The worker doesn't
    // serve traffic but it does sign + verify against Stripe and decrypt
    // TOTP secrets on demand (PR #15 refund flow uses the audit log path,
    // which the api binary owns; the worker just needs Stripe).
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
        sweep_interval_secs = SWEEP_INTERVAL.as_secs(),
        "starting worker binary",
    );

    let pool = storage::build_pool(&config)
        .await
        .context("build postgres pool")?;
    // Migrations run from api boot; the worker just consumes the schema.
    // Re-running migrate::run here would be safe but is unnecessary.

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
        pool,
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
    );

    let mut shutdown = ShutdownSignal::install();
    let mut ticker = interval(SWEEP_INTERVAL);
    // First tick fires immediately (default `interval` behavior) — run a
    // sweep at boot so a restart catches up any backlog without waiting
    // five minutes.

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                run_sweep(&state).await;
            }
            _ = shutdown.recv() => {
                tracing::info!("worker shutting down cleanly");
                return Ok(());
            }
        }
    }
}

async fn run_sweep(state: &AppState) {
    match reconcile_stripe_events(state, RECONCILE_MIN_AGE_SECS, SWEEP_LIMIT).await {
        Ok(n) if n > 0 => tracing::info!(redriven = n, "reconcile sweep done"),
        Ok(_) => tracing::debug!("reconcile sweep: no candidates"),
        Err(e) => tracing::error!(error = ?e, "reconcile sweep failed"),
    }
    match fetch_invoice_pdfs(state, SWEEP_LIMIT).await {
        Ok(n) if n > 0 => tracing::info!(uploaded = n, "invoice-pdf sweep done"),
        Ok(_) => tracing::debug!("invoice-pdf sweep: no candidates"),
        Err(e) => tracing::error!(error = ?e, "invoice-pdf sweep failed"),
    }
}

struct ShutdownSignal {
    sigterm: tokio::signal::unix::Signal,
    sigint: tokio::signal::unix::Signal,
}

impl ShutdownSignal {
    fn install() -> Self {
        Self {
            sigterm: signal(SignalKind::terminate()).expect("install SIGTERM handler"),
            sigint: signal(SignalKind::interrupt()).expect("install SIGINT handler"),
        }
    }
    async fn recv(&mut self) {
        tokio::select! {
            _ = self.sigterm.recv() => tracing::info!("received SIGTERM"),
            _ = self.sigint.recv() => tracing::info!("received SIGINT"),
        }
    }
}
