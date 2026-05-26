//! Persistence layer.
//!
//! BACKEND.md §1.6 + §2: the only crate that uses `sqlx::query!` macros. PR
//! #2 ships pool init, migrate-runner, and the DB readiness check. PR #3
//! adds `UsersRepo` + `SessionsRepo`. Repos for other tables land in their
//! respective PRs.

pub mod audit;
pub mod contact_messages;
pub mod course_progress;
pub mod downloads;
pub mod email_verifications;
pub mod entitlements;
pub mod invoices;
pub mod leads;
pub mod migrate;
pub mod notifications;
pub mod orders;
pub mod plans;
pub mod pool;
pub mod products;
pub mod readiness;
pub mod sessions;
pub mod stripe_events;
pub mod subscriptions;
pub mod users;

pub use audit::{AuditEntry, AuditError, AuditRepo};
pub use contact_messages::{ContactError, ContactMessage, ContactRepo};
pub use course_progress::{CourseProgressError, CourseProgressRepo, LessonProgress};
pub use downloads::{
    DownloadCatalogEntry, DownloadGrant, DownloadGrantsRepo, DownloadsCatalogRepo, DownloadsError,
};
pub use email_verifications::{
    EmailVerificationsError, EmailVerificationsRepo, PendingVerification, VerificationKind,
};
pub use entitlements::{
    generate_license_key, Enrollment, EnrollmentsRepo, EntitlementsError, IssuedLicense, License,
    LicensesRepo,
};
pub use invoices::{Invoice, InvoicesError, InvoicesRepo};
pub use leads::{Lead, LeadsError, LeadsRepo};
pub use notifications::{
    channels_for_kind, Notification, NotificationPayload, NotificationPreferences,
    NotificationPreferencesRepo, NotificationsError, NotificationsRepo, PreferencesPatch,
};
pub use orders::{CartSnapshotLine, NewOrderItem, Order, OrderItemRow, OrdersError, OrdersRepo};
pub use plans::{PlansError, PlansRepo, SubscriptionPlan};
pub use pool::{build_pool, PoolError};
pub use products::{Product, ProductsError, ProductsRepo};
pub use sessions::{ActiveSession, SessionListItem, SessionsError, SessionsRepo};
pub use stripe_events::{EventClaim, StoredEvent, StripeEventsError, StripeEventsRepo};
pub use subscriptions::{Subscription, SubscriptionsError, SubscriptionsRepo};
pub use users::{User, UserPatch, UsersError, UsersRepo};
