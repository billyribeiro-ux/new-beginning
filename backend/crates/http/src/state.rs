//! `AppState` — handler-shared services.
//!
//! BACKEND.md §6 specifies the final shape with repos, stripe client, r2,
//! mailer, etc. PR #3 adds: users/sessions repos, the Argon2id semaphore,
//! the login-attempt limiter, and the secrets (cookie HMAC key, password
//! pepper).

use common::alerts::AlertSink;
use common::config::Config;
use common::readiness::{DynReadinessCheck, ReadinessRegistry};
use r2_client::ObjectStore;
use secrecy::SecretString;
use sqlx::PgPool;
use std::sync::Arc;
use storage::{
    AuditRepo, ContactRepo, CourseProgressRepo, DownloadGrantsRepo, DownloadsCatalogRepo,
    EmailVerificationsRepo, EnrollmentsRepo, InvoicesRepo, LeadsRepo, LicensesRepo,
    NotificationPreferencesRepo, NotificationsRepo, OrdersRepo, PlansRepo, ProductsRepo,
    SessionsRepo, StripeEventsRepo, SubscriptionsRepo, UsersRepo,
};
use stripe_client::StripeApi;
use tokio::sync::Semaphore;

use crate::middleware::rate_limit::RateLimiterSet;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub alerts: Arc<dyn AlertSink>,
    pub readiness: ReadinessRegistry,
    /// Raw pool for ad-hoc transactions that span multiple repos (e.g.
    /// the Stripe webhook's mark-processed-in-tx flow, BACKEND.md §8.3).
    /// Cheap to clone — `PgPool` is `Arc`-internal.
    pub db: PgPool,
    pub users: UsersRepo,
    pub sessions: SessionsRepo,
    pub email_verifications: EmailVerificationsRepo,
    pub products: ProductsRepo,
    pub plans: PlansRepo,
    pub leads: LeadsRepo,
    pub contact: ContactRepo,
    pub orders: OrdersRepo,
    pub stripe_events: StripeEventsRepo,
    pub enrollments: EnrollmentsRepo,
    pub licenses: LicensesRepo,
    pub invoices: InvoicesRepo,
    pub subscriptions: SubscriptionsRepo,
    pub audit: AuditRepo,
    pub downloads_catalog: DownloadsCatalogRepo,
    pub download_grants: DownloadGrantsRepo,
    pub course_progress: CourseProgressRepo,
    pub notifications: NotificationsRepo,
    pub notification_prefs: NotificationPreferencesRepo,

    pub stripe: Arc<dyn StripeApi>,
    pub r2: Arc<dyn ObjectStore>,
    /// HMAC secret for verifying Stripe webhook signatures (BACKEND.md §8.3).
    pub stripe_webhook_secret: SecretString,

    /// Argon2id concurrency cap. BACKEND.md §7.2: every hash/verify call
    /// acquires a permit before invoking `spawn_blocking` so a burst of
    /// concurrent logins cannot OOM the host on m = 64 MiB.
    pub hash_semaphore: Arc<Semaphore>,

    /// `AUTH_COOKIE_KEY` — HMAC key for the session cookie's signature.
    pub cookie_key: SecretString,

    /// `AUTH_PASSWORD_PEPPER` — Argon2id `secret` parameter.
    pub password_pepper: SecretString,

    /// `AUTH_TOTP_KEY` — 32-byte key for XChaCha20-Poly1305 encryption of
    /// TOTP secrets at rest (BACKEND.md §7.3).
    pub totp_key: SecretString,

    /// Per-IP rate limiter for every bucket BACKEND.md §12 defines
    /// (login, public read, lead capture, contact). Single-instance only
    /// (BACKEND.md §23 risk #2); a Redis-backed impl can drop in here
    /// without touching call sites.
    pub limiter: RateLimiterSet,
}

impl AppState {
    /// Construct a state with the slice we have today. Subsequent PRs add
    /// repos/clients as builders or direct fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: Arc<Config>,
        alerts: Arc<dyn AlertSink>,
        db: PgPool,
        users: UsersRepo,
        sessions: SessionsRepo,
        email_verifications: EmailVerificationsRepo,
        products: ProductsRepo,
        plans: PlansRepo,
        leads: LeadsRepo,
        contact: ContactRepo,
        orders: OrdersRepo,
        stripe_events: StripeEventsRepo,
        enrollments: EnrollmentsRepo,
        licenses: LicensesRepo,
        invoices: InvoicesRepo,
        subscriptions: SubscriptionsRepo,
        audit: AuditRepo,
        downloads_catalog: DownloadsCatalogRepo,
        download_grants: DownloadGrantsRepo,
        course_progress: CourseProgressRepo,
        notifications: NotificationsRepo,
        notification_prefs: NotificationPreferencesRepo,
        stripe: Arc<dyn StripeApi>,
        r2: Arc<dyn ObjectStore>,
        cookie_key: SecretString,
        password_pepper: SecretString,
        totp_key: SecretString,
        stripe_webhook_secret: SecretString,
    ) -> Self {
        Self {
            config,
            alerts,
            readiness: ReadinessRegistry::new(),
            db,
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
            stripe_webhook_secret,
            hash_semaphore: Arc::new(Semaphore::new(8)),
            cookie_key,
            password_pepper,
            totp_key,
            limiter: RateLimiterSet::default(),
        }
    }

    pub fn with_readiness_check(mut self, check: DynReadinessCheck) -> Self {
        self.readiness = self.readiness.with(check);
        self
    }
}
