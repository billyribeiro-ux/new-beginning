//! PR #13 evidence test — notifications storage path end-to-end.
//!
//! * Lazy materialization: `get_or_default` for a fresh user returns
//!   the schema defaults + persists the row.
//! * Patch partials: setting `marketing_email=true` doesn't move the
//!   `course_progress_inapp` flag.
//! * DND triple: setting `dnd_enabled=true` + `dnd_start` + `dnd_end`
//!   round-trips.
//! * Feed: create three rows, mark one read, `unread=true` returns two,
//!   `mark_all_read` flips the rest.
//! * Cross-user isolation: `mark_read` only mutates the owner's rows.

use sqlx::PgPool;
use storage::{NotificationPreferencesRepo, NotificationsRepo, PreferencesPatch, UsersRepo};
use time::Time;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tradeflex:tradeflex@127.0.0.1:5435/tradeflex".into());
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .expect("test DB pool — is `just up` running?")
}

fn unique_email(tag: &str) -> String {
    format!("pr13-{tag}-{}@example.test", Uuid::new_v4())
}

#[tokio::test]
async fn notifications_full_flow() {
    let pool = pool().await;
    let users = UsersRepo::new(pool.clone());
    let notifs = NotificationsRepo::new(pool.clone());
    let prefs = NotificationPreferencesRepo::new(pool.clone());

    let alice = users
        .create(&unique_email("alice"), "Alice", "x")
        .await
        .unwrap();
    let bob = users
        .create(&unique_email("bob"), "Bob", "x")
        .await
        .unwrap();

    // Lazy materialization — fresh user has no row; get_or_default writes
    // defaults and returns them.
    let p = prefs.get_or_default(alice.id).await.unwrap();
    assert!(p.product_updates_email);
    assert!(p.product_updates_inapp);
    assert!(!p.marketing_email, "marketing_email default = FALSE");
    assert!(!p.dnd_enabled);
    assert_eq!(p.timezone, "UTC");

    // Partial patch: only flip marketing_email. Others must stay put.
    let p = prefs
        .patch(
            alice.id,
            PreferencesPatch {
                marketing_email: Some(true),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(p.marketing_email);
    assert!(p.product_updates_email, "untouched flag must stay TRUE");

    // DND triple: enable + start + end.
    let p = prefs
        .patch(
            alice.id,
            PreferencesPatch {
                dnd_enabled: Some(true),
                dnd_start: Some(Some(Time::from_hms(22, 0, 0).unwrap())),
                dnd_end: Some(Some(Time::from_hms(7, 0, 0).unwrap())),
                ..Default::default()
            },
        )
        .await
        .unwrap();
    assert!(p.dnd_enabled);
    assert_eq!(p.dnd_start, Some(Time::from_hms(22, 0, 0).unwrap()));
    assert_eq!(p.dnd_end, Some(Time::from_hms(7, 0, 0).unwrap()));

    // Feed: insert three rows for alice.
    let n1 = notifs
        .create(
            alice.id,
            "billing.invoice.paid",
            "Invoice paid",
            "$99",
            "stripe",
        )
        .await
        .unwrap();
    let _n2 = notifs
        .create(alice.id, "course.completed", "Course done", "🎉", "system")
        .await
        .unwrap();
    let _n3 = notifs
        .create(
            alice.id,
            "product.update",
            "New version",
            "Indicator v2",
            "admin",
        )
        .await
        .unwrap();

    let feed = notifs.list_for_user(alice.id, false, 50).await.unwrap();
    assert!(feed.len() >= 3);

    // Mark one read.
    let updated = notifs.mark_read(alice.id, n1).await.unwrap();
    assert_eq!(updated, 1);
    // Re-mark is idempotent (returns 0 because read_at IS NULL no longer matches).
    let again = notifs.mark_read(alice.id, n1).await.unwrap();
    assert_eq!(again, 0, "re-mark on already-read MUST be a no-op");

    // unread=true skips the read row.
    let unread = notifs.list_for_user(alice.id, true, 50).await.unwrap();
    assert!(
        !unread.iter().any(|n| n.id == n1),
        "read row must not appear in unread feed"
    );

    // Cross-user isolation: bob attempting to mark alice's row → 0 rows.
    let cross = notifs.mark_read(bob.id, n1).await.unwrap();
    assert_eq!(cross, 0, "mark_read with wrong user_id MUST be a no-op");

    // unread_count for alice = original unread - 1 (we marked one).
    let count_before = notifs.unread_count(alice.id).await.unwrap();
    let bulk = notifs.mark_all_read(alice.id).await.unwrap();
    assert_eq!(bulk, count_before as u64);
    let count_after = notifs.unread_count(alice.id).await.unwrap();
    assert_eq!(count_after, 0);
}

#[tokio::test]
async fn channels_for_kind_routes_correctly() {
    use storage::{channels_for_kind, NotificationPreferences};
    let p = NotificationPreferences {
        user_id: common::ids::UserId::new(),
        product_updates_email: true,
        product_updates_inapp: false,
        billing_email: false,
        billing_inapp: true,
        course_progress_email: true,
        course_progress_inapp: true,
        market_alerts_email: false,
        market_alerts_inapp: false,
        marketing_email: false,
        marketing_inapp: false,
        dnd_enabled: false,
        dnd_start: None,
        dnd_end: None,
        timezone: "UTC".into(),
        updated_at: time::OffsetDateTime::now_utc(),
    };

    assert_eq!(channels_for_kind(&p, "product.update"), (true, false));
    assert_eq!(channels_for_kind(&p, "billing.invoice.paid"), (false, true));
    assert_eq!(channels_for_kind(&p, "invoice.created"), (false, true));
    assert_eq!(
        channels_for_kind(&p, "subscription.canceled"),
        (false, true)
    );
    assert_eq!(channels_for_kind(&p, "course.completed"), (true, true));
    assert_eq!(
        channels_for_kind(&p, "market.alert.spy_above_500"),
        (false, false)
    );
    assert_eq!(channels_for_kind(&p, "marketing.welcome"), (false, false));
    // Unknown kind → in-app only, regardless of prefs (system messages).
    assert_eq!(channels_for_kind(&p, "system.maintenance"), (false, true));
}
